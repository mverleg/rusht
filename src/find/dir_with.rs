use ::std::fs;
use ::std::fs::DirEntry;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::log::debug;
use ::log::trace;
use ::regex::Regex;
use ::smallvec::{smallvec, SmallVec};
use log::{error, warn};

use crate::filter::{Keep, Order as UniqueOrder, unique_prefix};
use crate::find::{DirWithArgs, PathModification};
use crate::find::OnErr;
use crate::find::Order;
use crate::find::Nested::StopOnMatch;

enum IsMatch {
    Include,
    Exclude,
    NoMatch,
}

fn validate_roots_unique(roots: &[PathBuf]) -> Result<(), String> {
    let unique_roots = unique_prefix(
        roots
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        UniqueOrder::SortAscending,
        Keep::First,
    );
    if unique_roots.len() < roots.len() {
        return Err(format!(
            "root directories (-r) overlap; unique ones are: {}",
            unique_roots.iter().join(", ")
        ));
    }
    Ok(())
}

type Dirs = SmallVec<[PathBuf; 2]>;

pub fn find_dir_with(args: DirWithArgs) -> Result<Vec<PathBuf>, String> {
    validate_roots_unique(&args.roots)?;
    let mut results = vec![];
    for root in &args.roots {
        debug!("searching root '{}'", root.to_str().unwrap());
        let mut matches = find_matching_dirs(root, &args, args.max_depth)?;
        if args.path_modification == PathModification::Relative {
            matches = matches
                .into_iter()
                .map(|pth| {
                    pth.strip_prefix(root)
                        .expect("failed to make path relative")
                        .to_path_buf()
                })
                .collect();
        }
        results.extend(matches);
    }
    if args.order == Order::SortAscending {
        results.sort_unstable();
    }
    Ok(results)
}

fn find_matching_dirs(
    parent: &Path,
    args: &DirWithArgs,
    depth_remaining: u32,
) -> Result<Dirs, String> {
    if depth_remaining == 0 {
        return Ok(smallvec![]);
    }
    let mut current_is_match = false;
    let mut results: Dirs = match is_parent_match(parent, &args.itself, &args.not_self) {
        IsMatch::Include => {
            let found = parent.to_path_buf();
            if args.nested == StopOnMatch {
                debug!(
                "found a match based on parent name: {}, not recursing deeper",
                parent.to_str().unwrap()
            );
                return Ok(smallvec![found]);
            }
            debug!(
                "found a match based on parent name: {}, searching deeper",
                parent.to_str().unwrap()
            );
            current_is_match = true;
            smallvec![found]
        }
        IsMatch::Exclude => return Ok(smallvec![]),
        IsMatch::NoMatch => smallvec![],
    };
    let content = read_content(parent, args.on_err)?;
    trace!(
        "found {} items in {}",
        content.len(),
        parent.to_str().unwrap()
    );
    // separate loop so as not to recurse when early-exit is enabled
    for sub in &content {
        if !current_is_match && is_content_match(sub, &args.files, &args.dirs) {
            let found = parent.to_path_buf();
            if args.nested == StopOnMatch {
                debug!(
                    "found a match based on child name: {}, not recursing deeper",
                    sub.to_str().unwrap()
                );
                return Ok(smallvec![found]);
            }
            debug!(
                "found a match based on parent name: {}, searching deeper",
                sub.to_str().unwrap()
            );
            current_is_match = true;
            results.push(found)
        }
    }
    for sub in content {
        if !sub.is_dir() {
            continue;
        }
        let found = find_matching_dirs(&sub, args, depth_remaining - 1)?;
        results.extend(found);
    }
    Ok(results)
}

fn read_content(parent: &Path, on_err: OnErr) -> Result<Dirs, String> {
    let content = read_dir_err_handling(parent, on_err)?;
    let mut subdirs = smallvec![];
    for entry in content {
        subdirs.push(entry.path().to_path_buf())
    }
    Ok(subdirs)
}

fn read_dir_err_handling(dir: &Path, on_err: OnErr) -> Result<SmallVec<[DirEntry; 2]>, String> {
    match fs::read_dir(dir) {
        Ok(res) => {
            let mut entries = smallvec![];
            for entry in res {
                match entry {
                    Ok(entry) => entries.push(entry),
                    Err(err) => match on_err {
                        OnErr::Ignore => {}
                        OnErr::Warn => eprintln!("failed to read an entry in '{}', err {}; continuing (use -x=a to abort)", dir.to_str().unwrap(), err),
                        OnErr::Abort => eprintln!("failed to read an entry in '{}', err {}; stopping", dir.to_str().unwrap(), err),
                    }
                }
            }
            Ok(entries)
        }
        Err(err) => match on_err {
            OnErr::Ignore => Ok(smallvec![]),
            OnErr::Warn => {
                eprintln!(
                    "failed to scan directory '{}', err {}; continuing (use -x=a to abort)",
                    dir.to_str().unwrap(),
                    err
                );
                Ok(smallvec![])
            }
            OnErr::Abort => Err(format!(
                "failed to scan directory '{}', err {}; stopping",
                dir.to_str().unwrap(),
                err
            )),
        },
    }
}

/// Check if the parent itself matches one of the patterns.
fn is_parent_match(dir: &Path, positive_patterns: &[Regex], negative_patterns: &Vec<Regex>) -> IsMatch {
    if positive_patterns.is_empty() && negative_patterns.is_empty() {
        return IsMatch::NoMatch;
    }
    if let Some(dir_name) = dir.file_name() {
        let dir_name = dir_name.to_str().unwrap();
        for re in negative_patterns {
            if re.is_match(dir_name) {
                return IsMatch::Exclude;
            }
        }
        for re in positive_patterns {
            if re.is_match(dir_name) {
                debug!("parent match: '{}' matches '{}'", dir_name, re);
                return IsMatch::Include;
            }
        }
    }
    IsMatch::NoMatch
}

/// Check if this content item is a match (which causes the parent to be flagged).
fn is_content_match(item: &Path, file_res: &Vec<Regex>, dir_res: &Vec<Regex>) -> bool {
    if file_res.is_empty() && dir_res.is_empty() {
        return false;
    }
    if let Some(item_name) = item.file_name() {
        let item_name = item_name.to_str().unwrap();
        for re in file_res {
            if re.is_match(item_name) && item.is_file() {
                debug!("match: '{}' matches '{}'", item_name, re);
                return true;
            }
        }
        for re in dir_res {
            if re.is_match(item_name) && item.is_dir() {
                debug!("match: '{}' matches '{}'", item_name, re);
                return true;
            }
        }
    }
    false
}
