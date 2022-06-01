use ::structopt::StructOpt;
use regex::Regex;

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
