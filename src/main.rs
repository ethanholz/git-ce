use std::{fmt, io, process::Command};

use clap::Parser;
use console::Term;
use dialoguer::{FuzzySelect, Input};
use git2::{Repository, StatusOptions};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {}

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
    let _args = Args::parse();
    if Repository::open(".").is_err() {
        println!("Not a git repo!");
        return;
    }

    match has_staged_changes() {
        Ok(val) => {
            if !val {
                println!("No staged changes!");
                return;
            }
        }
        Err(err) => panic!("Error: {}", err),
    }

    let selections = &[
        "feat", "fix", "chore", "ci", "docs", "style", "refactor", "perf", "test",
    ];
    let term = Term::stdout();
    let mut commit = Commit::default();

    let commit_types = FuzzySelect::new()
        .with_prompt("Select a commit type")
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();
    match commit_types {
        Some(commit_types) => commit.commit_type = selections[commit_types].to_string(),
        None => return,
    };

    term.clear_last_lines(2).unwrap();
    term.flush().unwrap();

    let bc: String = Input::new()
        .with_prompt("Breaking changes")
        .default("".to_string())
        .interact()
        .unwrap();
    if bc.is_empty() {
        commit.breaking = Some(bc)
    } else {
        commit.breaking = None
    }

    term.clear_last_lines(1).unwrap();
    term.flush().unwrap();

    commit.message = Input::new()
        .with_prompt(&commit.commit_type)
        .interact()
        .unwrap();
    let built: String = format!("{}", commit);
    print!("{}", built);

    let _ = make_commit_shell(&built);
}

fn has_staged_changes() -> Result<bool, git2::Error> {
    let repo = Repository::open(".")?;
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
