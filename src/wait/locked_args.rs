use ::std::time::Duration;

use ::clap::Parser;
use ::parse_duration0::parse as parse_dur;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "locked",
    about = "Do not start a command until a given lock is released."
)]
pub struct LockedArgs {
    #[arg(short = 'f', long = "lock-key", default_value = "%{pwd}.lock")]
    /// The key to use for the lock. Only other commands with the same key are blocked. Can use %{pwd} and %{cmd} placeholders. Defaults to current directory.
    //TODO @mverleg: impl
    pub lock_key: String,
    #[arg(value_parser = parse_dur, short = 't', long = "timeout", default_value = "15 min")]
    /// Duration after which the waiting stops and the command fails. E.g. \"30 min\" or \"1 day -1 hour\".
    //TODO @mverleg: impl
    pub timeout: Duration,
    #[arg(short = 'p', long = "progress")]
    /// Show an indicator that we are still waiting, what is running, and how frequently we are checking.
    //TODO @mverleg: impl
    pub show_progress: bool,
    #[arg(short = 'r', long = "read")]
    /// Mark the current process as a reader instead of a writer. Multiple readers may hold the lock simultaneously. The process should not make any changes.
    //TODO @mverleg: impl
    pub read: bool,
    #[arg(short = 's', long = "show")]
    /// Instead of running a command, show the command(s) that currently hold the lock.
    //TODO @mverleg: impl
    pub show: bool,
    #[arg(long = "unlock")]
    /// Instead of running a command, remove the current lockfile. Should only be used if you are confident that the lock is held incorrectly.
    //TODO @mverleg: impl
    pub unlock: bool,
    #[command(subcommand)]
    //TODO @mverleg: impl
    pub cmd: CommandArgs,
}
//TODO @mverleg: allow multiple readers or one writer

#[test]
fn test_cli_args() {
    LockedArgs::try_parse_from(&["cmd", "-t=5 min", "ls"]).unwrap();
}
