use ::std::path::Path;

use ::git2::Repository;
use git2::DiffOptions;

pub fn git_master_base() {
    //git base-cmt HEAD || git rev-list --max-parents=0 HEAD
}

pub fn git_affected_files_commit(dir: &Path) -> Result<Vec<String>, String> {
    let repo = Repository::open(dir)
        .map_err(|err| format!("failed to read git repository at {}, err {}", dir.to_string_lossy(), err))?;
    let head_tree = repo.head().unwrap().peel_to_commit().unwrap().tree().unwrap();
        // .iter()
        // .map(|entry| entry.name().expect("non-utf8 filename").to_owned())
        // .collect::<Vec<_>>();
    let head_parent_tree = repo.head().unwrap().peel_to_commit().unwrap().parent(0).unwrap().tree().unwrap();
    let diff = repo.diff_tree_to_tree(Some(&head_tree), Some(&head_parent_tree), None).unwrap();
    for delta in diff.deltas() {
        eprintln!("{}", &delta.old_file().path().unwrap().to_string_lossy());
        if &delta.old_file().path() != &delta.new_file().path() {
            eprintln!("{}", &delta.new_file().path().unwrap().to_string_lossy());
        }
    }
        // .map_err(|err| format!("could not get git head commit at {}, err {}", dir.to_string_lossy(), err))?;
    // let files = repo.commi(repo.head(), repo.head().)
    //     .map_err(|err| format!("couldn't determine diff for head commit"))?;
    //diff-tree --no-commit-id --name-only -r HEAD;
    Ok(vec![])
}

pub fn git_affected_files_uncommitted() {
    //diff --name-only HEAD;
}

pub fn git_affected_files_branch() {
    //let base = git master-base;
    //git diff-tree --no-commit-id --name-only -r base;
}

#[cfg(test)]
mod tests {
    //TODO @mverleg: TEMPORARY! REMOVE THIS!
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn test_add() {
        git_affected_files_commit(&PathBuf::from("/Users/mverleg/data/goat")).unwrap();
    }
}