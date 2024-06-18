use std::{fmt, io, process::Command};

use clap::command;
use console::Term;
use dialoguer::{FuzzySelect, Input};
use git2::{Repository, StatusOptions};

#[derive(Default)]
struct Commit {
    commit_type: String,
    message: String,
    breaking: Option<String>,
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap();
    let _ = command!().get_matches();
    let repo = match Repository::discover(cwd) {
        Ok(repo) => repo,
        Err(_err) => {
            println!("Not a git repo!");
            return;
        }
    };
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
    //
    let selections = &[
        "feat", "fix", "chore", "ci", "docs", "style", "refactor", "perf", "test",
    ];
    let term = Term::stdout();
    let mut commit = Commit::default();

    let _ = ctrlc::set_handler(move || {
        let term = Term::stdout();
        let _ = term.show_cursor();
        term.clear_to_end_of_screen().unwrap();
        term.flush().unwrap();
    });

    let commit_types_result = FuzzySelect::new()
        .with_prompt("Select a commit type")
        .default(0)
        .items(&selections[..])
        .interact_on_opt(&term);
    match commit_types_result {
        Ok(commit_types) => {
            match commit_types {
                Some(commit_types) => commit.commit_type = selections[commit_types].to_string(),
                None => return,
            };
        }
        Err(_) => return,
    };
    term.clear_last_lines(1).unwrap();
    term.flush().unwrap();

    if parsed_scopes.len() == 1 {
        let scope = Input::new()
            .with_prompt("Scope")
            .default("".to_string())
            .interact_text();
        match scope {
            Ok(scope) => {
                if !scope.is_empty() {
                    commit.commit_type = format!("{}({})", commit.commit_type, scope);
                }
            }
            Err(_) => return,
        };
        term.clear_last_lines(1).unwrap();
        term.flush().unwrap();
    } else {
        let scope = FuzzySelect::new()
            .with_prompt("Scope")
            .default(0)
            .items(&parsed_scopes[..])
            .interact_on_opt(&term);
        match scope {
            Ok(scope) => {
                match scope {
                    Some(scope) => {
                        commit.commit_type =
                            format!("{}({})", commit.commit_type, parsed_scopes[scope]);
                    }
                    None => return,
                };
            }
            Err(_) => return,
        };
        term.clear_last_lines(1).unwrap();
        term.flush().unwrap();
    }

    let bc_result = Input::new()
        .with_prompt("Breaking changes")
        .default("".to_string())
        .interact_text();
    match bc_result {
        Ok(bc) => {
            commit.breaking = if bc.is_empty() { None } else { Some(bc) };
        }

        Err(_) => return,
    };

    term.clear_last_lines(1).unwrap();
    term.flush().unwrap();

    let message_result = Input::new()
        .with_prompt(&commit.commit_type)
        .interact_text();

    match message_result {
        Ok(message) => commit.message = message,
        Err(_) => return,
    }
    let built: String = format!("{}", commit);
    print!("{}", built);
    term.clear_last_lines(1).unwrap();
    term.flush().unwrap();

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

    #[test]
    fn test_print_commit() {
        let commit = super::Commit {
            commit_type: "feat".to_string(),
            message: "test".to_string(),
            breaking: None,
        };
        assert_eq!("feat: test", format!("{}", commit));
    }

    #[test]
    fn test_print_commit_breaking() {
        let commit = super::Commit {
            commit_type: "feat".to_string(),
            message: "test".to_string(),
            breaking: Some("breaking".to_string()),
        };
        assert_eq!(
            "feat!: test\n\nBREAKING CHANGE: breaking",
            format!("{}", commit)
        );
    }
}
