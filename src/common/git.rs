#![allow(unused)] //TODO @mverleg: TEMPORARY! REMOVE THIS!

use ::std::collections::HashSet;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::Instant;

use ::git2::Repository;
use ::log::debug;
use ::log::warn;

pub fn git_head_ref(dir: &Path) -> Result<String, String> {
    let repo = Repository::open(dir).map_err(|err| {
        format!(
            "failed to read git repository at {}, err {}",
            dir.to_string_lossy(),
            err
        )
    })?;
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    Ok(head.id().to_string())
}

pub fn git_master_base() {
    //git base-cmt HEAD || git rev-list --max-parents=0 HEAD
}

/// Returns changed and deleted files (separately) in head
pub fn git_affected_files_head(dir: &Path) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
    let t0 = Instant::now();
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
    let mut changed_files = HashSet::new();
    for delta in diff.deltas() {
        // Only add the new files, because old ones are either the same or don't exist anymore
        if let Some(pth) = delta.new_file().path() {
            changed_files.insert(pth.to_path_buf());
        }
    }
    let mut deleted_files = HashSet::new();
    for delta in diff.deltas() {
        // Only add the new files, because old ones are either the same or don't exist anymore
        if let Some(pth) = delta.old_file().path() {
            if !changed_files.contains(pth) {
                deleted_files.insert(pth.to_path_buf());
            }
        }
    }
    let duration = t0.elapsed().as_millis();
    if duration > 200 {
        warn!("git_affected_files_head slow, took {} ms", duration);
    } else {
        debug!("git_affected_files_head took {} ms", duration);
    }
    //TODO @mverleg: ^ this (hopefully) works for the specific commit, but when combining multiple commits, the files don't necessarily exist anymore at the end
    Ok((changed_files, deleted_files))
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
    use std::path::PathBuf;

    //TODO @mverleg: TEMPORARY! REMOVE THIS!
        use super::*;

    #[test]
    fn test_add() {
        git_affected_files_head(&PathBuf::from("/Users/mverleg/data/goat")).unwrap();
    }
}
