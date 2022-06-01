use ::std::fs;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::regex::Regex;
use ::structopt::StructOpt;
use ::ustr::Ustr;

use crate::find::unique::Keep;
use crate::find::unique::Order as UniqueOrder;
use crate::find::unique_prefix;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "dir_with",
    about = "Find directories that contain certain files or directories."
)]
pub struct DirWithArgs {
    #[structopt(short = "l", long, help = "Maximum directory depth to recurse into")]
    pub max_depth: Option<u32>,
    #[structopt(parse(from_flag = Order::from_is_sorted), short = "s", long = "sort", help = "Sort the results alphabetically")]
    pub order: Order,
    #[structopt(parse(from_flag = Nested::from_do_nested), short = "n", long = "nested", help = "Keep recursing even if a directory matches")]
    pub nested: Nested,
    #[structopt(parse(try_from_str = root_parser), short = "r", long = "root", required = true, default_value = ".", help = "Root directories to start searching from (multiple allowed)")]
    pub roots: Vec<PathBuf>,
    #[structopt(short = "f", long = "file", help = "File pattern that must exist in the directory to match")]
    pub files: Vec<Regex>,
    #[structopt(short = "d", long = "dir", help = "Subdirectory pattern that must exist in the directory to match")]
    pub dirs: Vec<Regex>,
    #[structopt(short = "i", long = "self", help = "Pattern for the directory itself for it to match")]
    pub itself: Vec<Regex>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Order {
    #[default]
    Preserve,
    SortAscending,
}

impl Order {
    fn from_is_sorted(is_sorted: bool) -> Self {
        if is_sorted {
            Order::SortAscending
        } else {
            Order::Preserve
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Nested {
    #[default]
    StopOnMatch,
    AlwaysRecurse,
}

impl Nested {
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

pub fn find_dir_with(args: DirWithArgs) -> Result<Vec<String>, String> {
    validate_roots_unique(&args.roots)?;

    unimplemented!()
}
