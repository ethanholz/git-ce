use dialoguer::{Editor, FuzzySelect, Input};
use git2::Repository;

fn main() {
    if Repository::open(".").is_err() {
        panic!("not a git repo");
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

    let built: String = format!("{}{}: commit\n", selections[commit_types], breaking_item);
    if let Some(rv) = Editor::new().edit(&built).unwrap() {
        println!("Your message: {}", rv);
    } else {
        println!("No message entered");
    }

    make_commit(&built).unwrap();
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
