use ::clap::Parser;

use ::regex::Regex;

#[derive(Parser, Debug)]
#[command(
    name = "between",
    about = "Select all lines between two matches"
)]
pub struct BetweenArgs {
    #[arg(short = 'f', long)]
    /// Start collecting lines when this expression matches
    pub from: Regex,
    #[arg(short = 't', long)]
    /// Stop collecting lines when this expression matches
    pub upto: Regex,
}

#[test]
fn test_cli_args() {
    BetweenArgs::try_parse_from(&["cmd", "--", "-f", ".*", "-t", "^END$"]).unwrap();
}
