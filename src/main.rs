use dialoguer::{Editor, FuzzySelect, Input};
use git2::{Repository, StatusOptions};

fn main() {
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

    let mut breaking_item = "";
    let selections = &[
        "feat", "fix", "chore", "ci", "docs", "style", "refactor", "perf", "test",
    ];
    let commit_types = FuzzySelect::new()
        .with_prompt("Pick your favorite flavor")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    println!("Selected {}", selections[commit_types]);

    let breaking: String = Input::new()
        .with_prompt("Breaking changes")
        .default("".to_string())
        .interact()
        .unwrap();
    if !breaking.is_empty() {
        breaking_item = "!";
    }

    let msg: String = Input::new()
        .with_prompt("Commit message")
        .interact()
        .unwrap();

    let mut built: String = format!("{}{}: {}", selections[commit_types], breaking_item, msg);
    if !breaking.is_empty() {
        built.push_str(&format!("\n\nBREAKING CHANGE: {}", breaking));
    }

    if let Some(rv) = Editor::new().edit(&built).unwrap() {
        make_commit(&rv).unwrap();
    } else {
        println!("No message entered");
    }
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

fn make_commit(message: &str) -> Result<(), git2::Error> {
    // Open the repository in the current directory
    let repo = Repository::open(".")?;

    // Create a new commit
    let tree_id = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?;
    let parent_commit = repo.find_commit(head.target().unwrap())?;
    let signature = repo.signature()?;

    let commit_id = repo.commit(
        Some("HEAD"),      // Update the current branch
        &signature,        // Author
        &signature,        // Committer
        message,           // Commit message
        &tree,             // Tree
        &[&parent_commit], // Parent commit(s)
    )?;

    // Print the commit ID
    println!("Commit created: {}", commit_id);

    Ok(())
}
