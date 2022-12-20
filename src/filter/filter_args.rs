use ::clap::Parser;

use ::regex::Regex;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "filter",
    about = "Run a test command for each line, keeping the file if the command succeeds"
)]
pub struct FilterArgs {
    #[arg(long)]
    /// Use a given regular expression that captures the value that is the input to the command. Uses the first capture group if any, or the whole match otherwise.
    pub by: Option<Regex>,
    #[arg(short = 'i', long)]
    /// Invert the command result, keeping all lines for which the command fails instead
    pub invert: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    FilterArgs::try_parse_from(&["cmd", "--help"]).unwrap();
}
