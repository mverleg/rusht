#![allow(unused)] //TODO @mverleg: TEMPORARY! REMOVE THIS!

use ::std::collections::HashSet;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::Instant;

use ::git2::Repository;
use ::log::debug;
use ::log::warn;
use git2::Error;

pub fn git_head_ref(dir: &Path) -> Result<String, String> {
    let repo = repo_open_ancestor(dir)?;
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    Ok(head.id().to_string())
}

fn repo_open_ancestor(deepest: &Path) -> Result<Repository, String> {
    let mut current = deepest;
    let mut msg = None;
    for i in 0..128 {
        match Repository::open(current) {
            Ok(repo) => return Ok(repo),
            Err(err) => msg.get_or_insert_with(|| err.to_string())
        };
        let Some(current) = current.parent() else {
            break
        };
    }
    Err(format!("failed to read git repository at {} or any of its parents, err {}",
        deepest.to_string_lossy(), msg.unwrap_or_else(|| "(no message)".to_owned())))
}

pub fn git_master_base() {
    //git base-cmt HEAD || git rev-list --max-parents=0 HEAD
}

/// Returns changed and deleted files (separately) in head
pub fn git_affected_files_head(dir: &Path) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
    let t0 = Instant::now();
    let repo = repo_open_ancestor(dir)?;
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
        let q = delta.new_file(); //TODO @mverleg: TEMPORARY! REMOVE THIS!
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
    use ::std::path::PathBuf;

    //TODO @mverleg: TEMPORARY! REMOVE THIS!
    use super::*;

    #[test]
    #[ignore] // doesn't work on CI
    fn test_git_repo() {
        git_affected_files_head(&PathBuf::from(".")).unwrap();
    }
}
