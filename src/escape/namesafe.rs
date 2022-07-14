use ::std::collections::HashSet;
use ::std::io;

use ::clap::StructOpt;
use ::log::debug;
use ::regex::Regex;

use crate::common::get_matches;

#[derive(StructOpt, Debug, Default)]
#[structopt(
    name = "namesafe",
    about = "Convert each line to a string that is safe for names (no whitespace or special characters, not too long)."
)]
pub struct NamesafeArgs {
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    NamesafeArgs::into_app().debug_assert()
}

pub fn namesafe(
    args: NamesafeArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut out_line_handler: impl FnMut(&str)
) {
    unimplemented!()
}
