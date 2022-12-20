use ::clap::Parser;

use ::regex::Regex;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[structopt(
    name = "filter",
    about = "Run a test command for each line, keeping the file if the command succeeds"
)]
pub struct FilterArgs {
    #[structopt(long)]
    /// Use a given regular expression that captures the value that is the input to the command. Uses the first capture group if any, or the whole match otherwise.
    pub by: Option<Regex>,
    #[structopt(short = 'i', long)]
    /// Invert the command result, keeping all lines for which the command fails instead
    pub invert: bool,
    #[structopt(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
use ::clap::FromArgMatches;
    FilterArgs::from_arg_matches().unwrap();
}
