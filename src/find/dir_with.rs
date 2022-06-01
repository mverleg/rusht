use ::std::fs;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::str::FromStr;
use std::fs::DirEntry;

use ::itertools::Itertools;
use ::log::debug;
use ::regex::Regex;
use ::smallvec::{SmallVec, smallvec};
use ::structopt::StructOpt;
use ::ustr::Ustr;

use crate::find::Nested::StopOnMatch;
use crate::find::unique::Keep;
use crate::find::unique::Order as UniqueOrder;
use crate::find::unique_prefix;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "dir_with",
    about = "Find directories that contain certain files or directories.",
    long_about = "Find directories that contain certain files or directories. Only supports utf8, sensible filenames.",
)]
pub struct DirWithArgs {
    #[structopt(short = "l", long, default_value = "10000", help = "Maximum directory depth to recurse into")]
    pub max_depth: u32,
    #[structopt(parse(from_flag = Order::from_is_sorted), short = "s", long = "sort", help = "Sort the results alphabetically")]
    pub order: Order,
    #[structopt(parse(from_flag = Nested::from_do_nested), short = "n", long = "nested", help = "Keep recursing even if a directory matches")]
    pub nested: Nested,
    #[structopt(short = "x", long = "on-error", help = "What to do when an error occurs: [w]arn, [a]bort or [i]gnore")]
    pub on_err: OnErr,
    #[structopt(parse(try_from_str = root_parser), short = "r", long = "root", required = true, default_value = ".", help = "Root directories to start searching from (multiple allowed)")]
    pub roots: Vec<PathBuf>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = "f", long = "file", help = "File pattern that must exist in the directory to match")]
    pub files: Vec<Regex>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = "d", long = "dir", help = "Subdirectory pattern that must exist in the directory to match")]
    pub dirs: Vec<Regex>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = "i", long = "self", help = "Pattern for the directory itself for it to match")]
    pub itself: Vec<Regex>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnErr {
    #[default]
    Warn,
    Abort,
    Ignore,
}

fn parse_full_str_regex(pattern: &str) -> Result<Regex, String> {
    let full_pattern = format!("^{}$", pattern);
    match Regex::new(&full_pattern) {
        Ok(re) => Ok(re),
        Err(err) => Err(format!("invalid file/dir pattern '{}'; it should be a valid regular expression, which will be wrapped inbetween ^ and $; err: {}", pattern, err)),
    }
}

impl FromStr for OnErr {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "w" | "warn" => OnErr::Warn,
            "a" | "abort" | "exit" | "stop" => OnErr::Abort,
            "i" | "ignore" | "silent" | "skip" => OnErr::Ignore,
            _ => return Err(format!("did not understand error handling strategy '{}', try '[w]arn', '[a]bort' or '[i]gnore'", value)),
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    #[default]
    Preserve,
    SortAscending,
}

impl Order {
    //TODO @mark: try from string
    fn from_is_sorted(is_sorted: bool) -> Self {
        if is_sorted {
            Order::SortAscending
        } else {
            Order::Preserve
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Nested {
    #[default]
    StopOnMatch,
    AlwaysRecurse,
}

impl Nested {
    //TODO @mark: try from string
    fn from_do_nested(do_nested: bool) -> Self {
        if do_nested {
            Nested::AlwaysRecurse
        } else {
            Nested::StopOnMatch
        }
    }
}

fn root_parser(root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(root);
    if fs::metadata(&path).is_err() {
        return Err(format!("did not find root '{}'", root))
    }
    Ok(path)
}

fn validate_roots_unique(roots: &[PathBuf]) -> Result<(), String> {
    let unique_roots = unique_prefix(
        roots.iter().map(|p| Ustr::from(p.to_string_lossy().as_ref())).collect(),
        UniqueOrder::SortAscending, Keep::First);
    if unique_roots.len() < roots.len() {
        return Err(format!("root directories (-r) overlap; unique ones are: {}", unique_roots.iter().join(", ")))
    }
    Ok(())
}

type Dirs = SmallVec<[PathBuf; 2]>;

pub fn find_dir_with(args: DirWithArgs) -> Result<Vec<PathBuf>, String> {
    validate_roots_unique(&args.roots)?;
    //TODO @mark: files
    //TODO @mark: dirs
    //TODO @mark: itself
    let mut results = vec![];
    for root in &args.roots {
        let matches = find_matching_dirs(root, &args, args.max_depth)?;
        results.extend(matches);
    }
    if args.order == Order::SortAscending {
        results.sort_unstable();
    }
    Ok(results)
}

fn find_matching_dirs(parent: &Path, args: &DirWithArgs, depth_remaining: u32) -> Result<Dirs, String> {
    if depth_remaining == 0 {
        return Ok(smallvec![])
    }
    let mut results: Dirs = SmallVec::new();
    for sub in read_subdirs(parent, args.on_err)? {
        if is_match(&sub, &args) {
            results.push(sub.canonicalize().expect("failed to create absolute path"));
            if args.nested == StopOnMatch {
                debug!("found a match: {}, not recursing deeper", sub.to_str().unwrap());
                continue;
            }
            debug!("found a match: {}, searching deeper", sub.to_str().unwrap());
        }
        let found = find_matching_dirs(&sub, args, depth_remaining - 1)?;
        results.extend(found);
    }
    Ok(results)
}

fn read_subdirs(parent: &Path, on_err: OnErr) -> Result<Dirs, String> {
    let content = read_dir_err_handling(parent, on_err)?;
    let mut subdirs = smallvec![];
    for entry in content {
        if entry.path().is_dir() {
            subdirs.push(entry.path().to_path_buf())
        }
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
        },
        Err(err) => match on_err {
            OnErr::Ignore => Ok(smallvec![]),
            OnErr::Warn => {
                eprintln!("failed to scan directory '{}', err {}; continuing (use -x=a to abort)", dir.to_str().unwrap(), err);
                Ok(smallvec![])
            },
            OnErr::Abort => return Err(format!("failed to scan directory '{}', err {}; stopping", dir.to_str().unwrap(), err)),
        }
    }
}

fn is_match(dir: &Path, args: &DirWithArgs) -> bool {
    for me in &args.itself {
        if me.is_match(dir.to_str().unwrap()) {
            return true
        }
    }
    false
}
