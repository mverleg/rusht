use ::clap::Parser;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "mon",
    about = "Log the command, the outcome, timings and play a sound."
)]
pub struct MonArgs {
    /// Do not show the command before running it.
    #[arg(short = 'c', long = "no-print-cmd")]
    pub no_print_cmd: bool,
    /// Only print command output if the command fails.
    #[arg(short = 'b', long = "no-output-on-success")]
    pub no_output_on_success: bool,
    /// Do not report timing, and suppress status line when successful.
    #[arg(short = 't', long)]
    pub no_timing: bool,
    /// Play a sound when the command succeeds.
    #[arg(short = 's', long = "ok-sound")]
    pub sound_success: bool,
    /// Play a sound when the command fails.
    #[arg(short = 'S', long = "fail-sound")]
    pub sound_failure: bool,
    /// Prefix each line. Can use '%{date}' and '%{time}' placeholders.
    #[arg(short = 'p', long)]
    pub prefix: Option<String>,
    /// Log command and timing to stdout instead of stderr
    #[arg(short = 'x', long)]
    pub use_stdout: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    MonArgs::try_parse_from(&["cmd", "ls"]).unwrap();
}
