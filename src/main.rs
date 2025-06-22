use std::{fmt, io, process::Command};

use clap::{command, Parser};
use git2::{Repository, StatusOptions};
use inquire::{Select, Text};
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::openai::{self, O3_MINI},
};

#[derive(Default)]
struct Commit {
    commit_type: String,
    message: String,
    breaking: Option<String>,
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            match &self.breaking {
                Some(_) => return write!(f, "{}!:", self.commit_type),
                None => return write!(f, "{}:", self.commit_type),
            }
        }
        match &self.breaking {
            Some(breaking) => write!(
                f,
                "{}!: {}\n\nBREAKING CHANGE: {}",
                self.commit_type, self.message, breaking
            ),
            None => write!(f, "{}: {}", self.commit_type, self.message),
        }
    }
}

#[derive(Parser)]
#[command(name = "git-ce")]
#[command(about = "Conventional commit helper")]
#[command(version = env!("CARGO_PKG_VERSION"))] // Uses version from Cargo.toml
struct Cli {
    /// Output git diff and status for LLM processing
    #[arg(long)]
    llm: bool,
}

fn get_commit_info(repo: &Repository) -> Result<String, git2::Error> {
    let mut output = String::new();

    // Get the staged diff (equivalent to git diff --cached)
    let head = repo.head()?.peel_to_tree()?;
    let index = repo.index()?;

    let diff = repo.diff_tree_to_index(Some(&head), Some(&index), None)?;

    // Collect the diff output
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin() {
            '+' | '-' | ' ' => output.push_str(&format!(
                "{}{}",
                line.origin(),
                std::str::from_utf8(line.content()).unwrap()
            )),
            _ => output.push_str(std::str::from_utf8(line.content()).unwrap()),
        }
        true
    })?;

    Ok(output)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap();
    let repo = match Repository::discover(cwd) {
        Ok(repo) => repo,
        Err(_err) => {
            println!("Not a git repo!");
            return;
        }
    };
    if cli.llm {
        let info = get_commit_info(&repo).unwrap();
        let prompt = format!(
            "Write me a conventional commit message using the following: {}",
            info
        );
        let openai_client = openai::Client::from_env();
        let model = openai_client.agent(O3_MINI).build();
        let response = model.prompt(prompt).await.expect("Failed to prompt");
        make_commit_shell(&response).unwrap();
        return;
    }
    let config = repo.config().unwrap();
    let scopes = config.multivar("ce.scope", None).unwrap();
    let mut parsed_scopes = vec!["".to_string()];
    let _ = scopes.for_each(|e| {
        if let Some(value) = e.value() {
            parsed_scopes.push(value.to_string());
        }
    });

    match has_staged_changes(&repo) {
        Ok(val) => {
            if !val {
                println!("No staged changes!");
                return;
            }
        }
        Err(err) => panic!("Error: {}", err),
    }

    let selections = vec![
        "feat", "fix", "chore", "ci", "docs", "style", "refactor", "perf", "test",
    ];
    let mut commit = Commit::default();
    let ans = Select::new("Select a commit type", selections).prompt();
    match ans {
        Ok(choice) => {
            commit.commit_type = choice.to_string();
        }
        Err(_) => return,
    }

    if parsed_scopes.len() == 1 {
        let scope = Text::new("Scope").prompt();
        match scope {
            Ok(scope) => {
                if !scope.is_empty() {
                    commit.commit_type = format!("{}({})", commit.commit_type, scope);
                }
            }
            Err(_) => return,
        }
    } else {
        let scope = Select::new("Scope", parsed_scopes).prompt();
        match scope {
            Ok(scope) => {
                if !scope.is_empty() {
                    commit.commit_type = format!("{}({})", commit.commit_type, scope);
                }
            }
            Err(_) => return,
        }
    }

    let bc_result = Text::new("Breaking changes").prompt();
    match bc_result {
        Ok(bc) => {
            commit.breaking = if bc.is_empty() { None } else { Some(bc) };
        }
        Err(_) => return,
    }

    let message_prompt = format!("{}", commit);
    let message = Text::new(message_prompt.as_str()).prompt();
    match message {
        Ok(message) => commit.message = message,
        Err(_) => return,
    }

    let built: String = format!("{}", commit);
    let _ = make_commit_shell(&built);
}

fn has_staged_changes(repo: &Repository) -> Result<bool, git2::Error> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false)
        .renames_head_to_index(true)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;
    let count = statuses.iter().find(|e| {
        e.status().is_index_new()
            || e.status().is_index_modified()
            || e.status().is_index_renamed()
            || e.status().is_index_typechange()
    });

    Ok(count.is_some())
}

// farm commits out to the shell to handle the editor
fn make_commit_shell(message: &str) -> Result<std::process::ExitStatus, io::Error> {
    let args = vec!["commit", "-m", message, "-e"];
    Command::new("git").args(args).status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_print_commit() {
        let commit = Commit {
            commit_type: "feat".to_string(),
            message: "test".to_string(),
            breaking: None,
        };
        assert_eq!("feat: test", format!("{}", commit));
    }

    #[test]
    fn test_print_commit_breaking() {
        let commit = Commit {
            commit_type: "feat".to_string(),
            message: "test".to_string(),
            breaking: Some("breaking".to_string()),
        };
        assert_eq!(
            "feat!: test\n\nBREAKING CHANGE: breaking",
            format!("{}", commit)
        );
    }

    #[test]
    fn test_has_staged_changes() {
        let temp = tempdir().unwrap();
        std::env::set_current_dir(&temp).unwrap();

        // Create a git repo
        let repo = Repository::init(&temp).unwrap();
        assert!(!has_staged_changes(&repo).unwrap());
        // Create a file
        let file_path = temp.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello").unwrap();

        // Add the file
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();

        assert!(has_staged_changes(&repo).unwrap());
    }
}
