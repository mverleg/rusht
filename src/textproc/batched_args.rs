use ::clap::Parser;
use regex::Regex;

use crate::common::CommandArgs;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "batched",
    about = "Split stdin into groups of N lines, and pass them to the command"
)]
pub struct BatchedArgs {
    /// Maximum number of items per batch (may be smaller, e.g. for last one, or due to filtering).
    #[arg(short = 'c', long, value_parser = clap::value_parser!(u32).range(2..))]
    pub batch_size: u32,
    /// Items having the same value for this regex are put in the same batch (up to the maximum size). Multiple values may be in the same batch.
    #[arg(long)]
    pub together: Option<Regex>,
    /// Items having the same value for this regex are put in separate batches (creating extra batches as necessary).
    #[arg(long, conflicts_with = "together")]
    pub apart: Option<Regex>,
    /// Allow different groups and remaining items in the same batch, if it can be done without extra splitting.
    #[arg(long)]
    pub mixed_groups: bool,
    /// Drop any items that do not match the regex, they are not included in any batch.
    #[arg(long)]
    pub drop_unmatched: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    BatchedArgs::try_parse_from(&["batched", "-c=2", "wc", "-l"]).unwrap();
    BatchedArgs::try_parse_from(&[
        "batched",
        "-c=2",
        "--apart",
        "nr([0-9]+)",
        "--mixed-groups",
        "implode",
    ])
    .unwrap();
    BatchedArgs::try_parse_from(&["batched", "-c=2", "--together", "^\\w+", "implode"]).unwrap();
}
