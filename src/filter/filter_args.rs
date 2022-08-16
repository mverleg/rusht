use ::clap::StructOpt;

use ::regex::Regex;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "filter",
    about = "Run a test command for each line, keeping the file if the command succeeds"
)]
pub struct FilterArgs {
    #[structopt(
        long,
        help = "Use a given regular expression that captures the value that is the input to the command. Uses the first capture group if any, or the whole match otherwise."
    )]
    pub by: Option<Regex>,
    #[structopt(
        short = 'i',
        long,
        help = "Invert the command result, keeping all lines for which the command fails instead"
    )]
    pub invert: bool,
    #[structopt(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    FilterArgs::into_app().debug_assert()
}
