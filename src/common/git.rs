#![allow(unused)] //TODO @mverleg: TEMPORARY! REMOVE THIS!

use ::std::path::Path;
use std::collections::HashSet;
use std::path::PathBuf;

use ::git2::Repository;

pub fn git_head_ref(dir: &Path) -> Result<String, String> {
    let repo = Repository::open(dir).map_err(|err| {
        format!(
            "failed to read git repository at {}, err {}",
            dir.to_string_lossy(),
            err
        )
    })?;
    let head = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap();
    Ok(head.id().to_string())
}

pub fn git_master_base() {
    //git base-cmt HEAD || git rev-list --max-parents=0 HEAD
}

pub fn git_affected_files_head(dir: &Path) -> Result<HashSet<PathBuf>, String> {
    let repo = Repository::open(dir).map_err(|err| {
        format!(
            "failed to read git repository at {}, err {}",
            dir.to_string_lossy(),
            err
        )
    })?;
    let head_tree = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .tree()
        .unwrap();
    // .iter()
    // .map(|entry| entry.name().expect("non-utf8 filename").to_owned())
    // .collect::<Vec<_>>();
    let mut changed_files = HashSet::new();
    let head_parent_tree = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .parent(0)
        .unwrap()
        .tree()
        .unwrap();
    let diff = repo
        .diff_tree_to_tree(Some(&head_tree), Some(&head_parent_tree), None)
        .unwrap();
    for delta in diff.deltas() {
        if let Some(pth) = delta.old_file().path() {
            changed_files.insert(pth.to_path_buf());
        }
        if let Some(pth) = delta.new_file().path() {
            if ! changed_files.contains(pth) {
                changed_files.insert(pth.to_path_buf());
            }
        }
    }
    Ok(changed_files)
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
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add() {
        git_affected_files_head(&PathBuf::from("/Users/mverleg/data/goat")).unwrap();
    }
}
