use ::clap::StructOpt;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "mon",
    about = "Log the command, the outcome, timings and play a sound."
)]
pub struct MonArgs {
    /// Do not show the command before running it
    #[structopt(short = 'c', long)]
    pub no_print_cmd: bool,
    /// Do not show timing and exit status of the command
    #[structopt(short = 't', long)]
    pub no_timing: bool,
    /// Do not play a sound when the command succeeds
    #[structopt(short = 's', long = "no-ok-sound")]
    pub no_sound_success: bool,
    /// Do not play a sound when the command fails
    #[structopt(short = 'S', long = "no-fail-sound")]
    pub no_sound_failure: bool,
    #[structopt(subcommand)]
    pub cmd: CommandArgs,
}
//TODO @mverleg: implement all that ^

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    MonArgs::into_app().debug_assert()
}