use ::std::fs;
use ::std::fs::DirEntry;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::log::debug;
use ::log::trace;
use ::regex::Regex;
use ::smallvec::{smallvec, SmallVec};

use crate::filter::{unique_prefix, Keep, Order as UniqueOrder};
use crate::find::Nested::StopOnMatch;
use crate::find::OnErr;
use crate::find::Order;
use crate::find::{DirWithArgs, PathModification};

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

pub fn find_dir_with(args: &DirWithArgs) -> Result<Vec<PathBuf>, String> {
    debug!("args = {:?}", args);
    validate_roots_unique(&args.roots)?;
    let mut results = if args.upwards {
        find_dir_with_upwards(args)
    } else {
        find_dir_with_downwards(args)
    }?;
    if args.order == Order::SortAscending {
        results.sort_unstable();
    }
    Ok(results)
}

pub fn find_dir_with_downwards(args: &DirWithArgs) -> Result<Vec<PathBuf>, String> {
    debug_assert!(!args.upwards);
    let mut results = vec![];
    for root in &args.roots {
        debug!("searching root '{}' downwards", root.to_str().unwrap());
        let mut matches = find_matching_dirs(root, &args, args.max_depth)?;
        if args.path_modification == PathModification::Relative {
            matches = make_relative(root, &mut matches);
        }
        results.extend(matches);
    }
    Ok(results)
}

pub fn find_dir_with_upwards(args: &DirWithArgs) -> Result<Vec<PathBuf>, String> {
    debug_assert!(args.upwards);
    let mut results = Vec::new();
    for root in args.roots.clone() {
        debug!("searching root '{}' upwards", root.to_str().unwrap());
        let mut depth = 1;
        let mut current = root.as_path();
        let mut depth_remaining = args.max_depth;
        while let Some(next) = current.parent() && depth_remaining > 0 {
            depth_remaining -= 1;
            current = next;

            // Use depth_remaining=0 here, because this is downward depths, not upward
            let mut matches = find_matching_dirs(&root, &args, 0)?;

            if args.path_modification == PathModification::Relative {
                matches = make_relative(&root, &mut matches);
            }
            results.extend(matches);
        }
        debug!("stopping for '{}' either because there is no parent or max depth {} is reached",
            current.to_string_lossy(), args.max_depth);
    }
    Ok(results)
}

fn make_relative(root: &PathBuf, matches: &mut Dirs) -> Dirs {
    matches.into_iter()
        .map(|pth| {
            pth.strip_prefix(root)
                .expect("failed to make path relative")
                .to_path_buf()
        })
        .collect()
}

/// Can be made non-recursive with depth_remaining=0
fn find_matching_dirs(
    parent: &Path,
    args: &DirWithArgs,
    depth_remaining: u32,
) -> Result<Dirs, String> {
    if depth_remaining == 0 {
        return Ok(smallvec![]);
    }
    let content = dir_listing(parent, args.on_err)?;
    let children_count_in_range = args.child_count_range.includes(content.len() as u32);
    let mut results: Dirs;
    let mut current_is_match = false;
    let parent_match = if_count_ok(
        children_count_in_range,
        is_parent_match(parent, &args.itself, &args.not_self),
    );
    results = match parent_match {
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
    trace!(
        "found {} items in {}",
        content.len(),
        parent.to_str().unwrap()
    );
    // separate loop so as not to recurse when early-exit is enabled
    for sub in &content {
        if current_is_match {
            continue;
        }
        let content_match = if_count_ok(
            children_count_in_range,
            is_content_match(
                sub,
                &args.files,
                &args.not_files,
                &args.dirs,
                &args.not_dirs,
            ),
        );
        match content_match {
            IsMatch::Include => {
                let found = parent.to_path_buf();
                if args.nested == StopOnMatch {
                    debug!(
                        "found a match based on child name: {}, not recursing deeper",
                        sub.to_str().unwrap()
                    );
                    return Ok(smallvec![found]);
                }
                debug!(
                    "found a match based on child name: {}, searching deeper",
                    sub.to_str().unwrap()
                );
                current_is_match = true;
                results.push(found)
            }
            IsMatch::Exclude => return Ok(smallvec![]),
            IsMatch::NoMatch => {}
        }
    }
    let has_positive_pattern =
        !args.itself.is_empty() || !args.files.is_empty() || !args.dirs.is_empty();
    if args.child_count_range.is_provided()
        && children_count_in_range
        && !has_positive_pattern
        && !current_is_match
    {
        //TODO @mverleg: need to make sure range isn't default
        debug!("selecting {} based on range {} because there were no positive patterns, and negative ones did not match",
            parent.to_str().unwrap(), args.child_count_range);
        results.push(parent.to_path_buf())
    }
    if depth_remaining <= 1 {
        return Ok(results)
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

fn if_count_ok(count_ok: bool, is_match: IsMatch) -> IsMatch {
    if !count_ok && matches!(is_match, IsMatch::Include) {
        return IsMatch::Exclude;
    }
    is_match
}

fn dir_listing(parent: &Path, on_err: OnErr) -> Result<Dirs, String> {
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
fn is_parent_match(
    dir: &Path,
    positive_patterns: &[Regex],
    negative_patterns: &Vec<Regex>,
) -> IsMatch {
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
fn is_content_match(
    item: &Path,
    positive_file_pattern: &Vec<Regex>,
    negative_file_pattern: &Vec<Regex>,
    positive_dir_pattern: &Vec<Regex>,
    negative_dir_pattern: &Vec<Regex>,
) -> IsMatch {
    if positive_file_pattern.is_empty()
        && negative_file_pattern.is_empty()
        && positive_dir_pattern.is_empty()
        && negative_dir_pattern.is_empty()
    {
        return IsMatch::NoMatch;
    }
    if let Some(item_name) = item.file_name() {
        let item_name = item_name.to_str().unwrap();
        for re in positive_file_pattern {
            if re.is_match(item_name) && item.is_file() {
                debug!("match: '{}' matches '{}'", item_name, re);
                return IsMatch::Include;
            }
        }
        for re in negative_file_pattern {
            if re.is_match(item_name) && item.is_file() {
                debug!("negative match: '{}' matches '{}'", item_name, re);
                return IsMatch::Exclude;
            }
        }
        for re in positive_dir_pattern {
            if re.is_match(item_name) && item.is_dir() {
                debug!("match: '{}' matches '{}'", item_name, re);
                return IsMatch::Include;
            }
        }
        for re in negative_dir_pattern {
            if re.is_match(item_name) && item.is_dir() {
                debug!("negative match: '{}' matches '{}'", item_name, re);
                return IsMatch::Exclude;
            }
        }
    }
    IsMatch::NoMatch
}
