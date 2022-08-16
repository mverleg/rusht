use ::std::fs;
use ::std::path::PathBuf;
use ::std::str::FromStr;

use ::clap::StructOpt;
use ::regex::Regex;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "dir_with",
    about = "Find directories that contain certain files or directories.",
    long_about = "Find directories that contain certain files or directories. Only supports utf8, sensible filenames."
)]
pub struct DirWithArgs {
    #[structopt(short = 'l', long, default_value = "10000")]
    /// Maximum directory depth to recurse into
    pub max_depth: u32,
    #[structopt(parse(from_flag = Order::from_is_sorted), short = 's', long = "sort")]
    /// Sort the results alphabetically
    pub order: Order,
    #[structopt(parse(from_flag = Nested::from_do_nested), short = 'n', long = "nested")]
    /// Keep recursing even if a directory matches
    pub nested: Nested,
    #[structopt(short = 'x', long = "on-error", default_value = "warn")]
    /// What to do when an error occurs: [w]arn, [a]bort or [i]gnore
    pub on_err: OnErr,
    #[structopt(parse(from_flag = PathModification::from_is_relative), short = 'z', long = "relative")]
    /// Results are relative to roots, instead of absolute
    pub path_modification: PathModification,
    #[structopt(parse(try_from_str = root_parser), short = 'r', long = "root", required = true, default_value = ".")]
    /// Root directories to start searching from (multiple allowed)
    pub roots: Vec<PathBuf>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'f', long = "file")]
    /// File pattern that must exist in the directory to match
    pub files: Vec<Regex>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'd', long = "dir")]
    /// Subdirectory pattern that must exist in the directory to match
    pub dirs: Vec<Regex>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'i', long = "self")]
    /// Pattern for the directory itself for it to match
    pub itself: Vec<Regex>,
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'F', long = "not-file")]
    /// Opposite of -f; directory only matches if this file pattern is NOT matched inside it
    pub not_files: Vec<Regex>,
    //TODO @mverleg: ^
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'D', long = "not-dir")]
    /// Opposite of -d, directory only matches if this directory pattern is NOT matched inside it
    pub not_dirs: Vec<Regex>,
    //TODO @mverleg: ^
    #[structopt(parse(try_from_str = parse_full_str_regex), short = 'I', long = "not-self")]
    /// Opposite of -i, directory only matches if its own name does NOT match this pattern
    pub not_self: Vec<Regex>,
    //TODO @mverleg: ^
    // #[structopt(parse(from_flag = Nested::from_do_nested), short = 'N', long = "exclude-not")]
    // Keep recursing even if a directory is negative-matched by -F/-D/-I
    // pub negative_nested: Nested,
    // //TODO @mverleg: ^
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    DirWithArgs::into_app().debug_assert()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PathModification {
    Relative,
    #[default]
    Canonical,
}

impl PathModification {
    fn from_is_relative(is_relative: bool) -> Self {
        if is_relative {
            PathModification::Relative
        } else {
            PathModification::Canonical
        }
    }
}

fn parse_full_str_regex(pattern: &str) -> Result<Regex, String> {
    let full_pattern = format!("^{}$", pattern);
    match Regex::new(&full_pattern) {
        Ok(re) => Ok(re),
        Err(err) => Err(format!("invalid file/dir pattern '{}'; it should be a valid regular expression, which will be wrapped inbetween ^ and $; err: {}", pattern, err)),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnErr {
    #[default]
    Warn,
    Abort,
    Ignore,
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
        return Err(format!("did not filter root '{}'", root));
    }
    Ok(path)
}
