use ::std::collections::HashSet;

use ::log::debug;
use ::structopt::StructOpt;
use ::ustr::Ustr;
use ::ustr::UstrSet;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "dir_with",
    about = "Find directories that contain certain files or directories."
)]
pub struct DirWithArgs {
    #[structopt(short = "l", long, help = "Maximum directory depth to recurse into")]
    pub max_depth: u32,
    #[structopt(parse(from_flag = Order::from_is_sorted), short = "s", long = "sort", help = "Sort the results alphabetically")]
    pub order: Order,
    #[structopt(parse(from_flag = Nested::from_do_nested), short = "n", long = "nested", help = "Keep recursing even if a directory matches")]
    pub nested: Nested,
    #[structopt(short = "f", long = "file", help = "File that must exist in the directory to match")]
    pub files: Vec<String>,
    #[structopt(short = "d", long = "dir", help = "Subdirectory that must exist in the directory to match")]
    pub dirs: Vec<String>,
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
