use ::clap;
use ::clap::Parser;

use crate::common::CommandArgs;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "batched",
    about = "Split stdin into groups of N lines, and pass them to the command",
)]
pub struct BatchedArgs {
    /// Maximum number of items per batch (last one may be smaller).
    #[arg(short = 'c', long, value_parser = clap::value_parser!(u32).range(2..))]
    pub batch_size: u32,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    BatchedArgs::try_parse_from(&["batched", "-c=1", "implode",]).unwrap();
}
