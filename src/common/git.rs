#![allow(unused)] //TODO @mverleg: TEMPORARY! REMOVE THIS!

use crate::common::Task;
use crate::common::VecWriter;
use ::base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ::base64::Engine;
use ::git2::Error;
use ::git2::Repository;
use ::log::debug;
use ::log::warn;
use ::num_cpus::get;
use ::sha2::Digest;
use ::sha2::Sha256;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::Instant;
use regex::Regex;
use sha2::digest::Update;

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
        let Some(par) = current.parent() else {
            break
        };
        current = par
    }
    Err(format!("failed to read git repository at {} or its parents, err {}",
        deepest.to_string_lossy(), msg.unwrap_or_else(|| "(no message)".to_owned())))
}

pub async fn git_master_base_ref(dir: &Path) -> Result<String, String> {
    let mut lines = git_shell_cmd(
        dir,
        vec!["merge-base".to_owned(), "origin/master".to_owned(), "HEAD".to_owned()],
        //TODO @mverleg: master?
        "getting git merge base",
    ).await?;
    if lines.len() != 1 {
        return Err(format!("unexpected response when getting git merge base: {}", lines.join("\\n")))
    }
    Ok(lines.pop().unwrap())
}

pub async fn git_repo_dir(dir: &Path) -> Result<String, String> {
    let mut lines = git_shell_cmd(
        dir,
        vec!["rev-parse".to_owned(), "--show-toplevel".to_owned()],
        "getting git repo root directory",
    ).await?;
    if lines.len() != 1 {
        return Err(format!("unexpected response when getting git repo root directory: {}", lines.join("\\n")))
    }
    Ok(lines.pop().unwrap())
}

pub async fn git_uncommitted_changes(dir: &Path) -> Result<Vec<String>, String> {
    let lines = git_shell_cmd(
        dir,
        vec!["status".to_owned(), "-v".to_owned(), "--porcelain".to_owned()],
        "finding uncommitted git changes",
    ).await?;
    Ok(lines.into_iter()
        .flat_map(|line| line.split_ascii_whitespace()
            .skip(1).next()
            .map(ToOwned::to_owned))
        .collect())
}

pub async fn git_stripped_diff(dir: &Path, rev: &str) -> Result<String, String> {
    // HOME= git diff 'HEAD^!' | grep -v @@ | grep -v index
    let mut hasher = Sha256::new();
    let re = Regex::new("^@@.*@@").unwrap();
    git_shell_cmd(
        dir,
        vec!["diff".to_owned(), format!("{rev}^!")],
        "getting git commit diff",
    ).await?.into_iter()
        .filter(|line| !line.starts_with("@@ "))
        .map(|line| if line.contains("@@") { re.replace(&line, "").to_string() } else { line })
        .for_each(|line| sha2::digest::Update::update(&mut hasher, line.as_bytes()));
    let hash_out = hasher.finalize();
    Ok(URL_SAFE_NO_PAD.encode(hash_out))
}

async fn git_shell_cmd(
    dir: &Path,
    git_args: Vec<String>,
    context_descr: &str
) -> Result<Vec<String>, String> {
    let mut output = VecWriter::new();
    let mut errors = VecWriter::new();
    let mut no_home = HashMap::new();
    no_home.insert("HOME".to_owned(), "".to_owned());
    let status = Task::new_with_env(
        "git".to_owned(),
        git_args,
        dir.to_owned(),
        None,
        no_home
    ).execute_with_stdout_nomonitor(
        &mut output,
        &mut errors
    ).await;
    if status.is_ok() {
        Ok(output.get())
    } else {
        Err(format!("error while {context_descr}: {}", errors.get().join("; ")))
    }
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
