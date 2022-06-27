use ::std::time::Duration;

use ::parse_duration::parse as parse_dur;
use ::clap::StructOpt;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "locked",
    about = "Do not start a command until a given lock is released."
)]
pub struct DirLockedArgs {
    #[structopt(
        short = 'f',
        long = "lock-key",
        default_value = "${pwd}.lock",
        help = "The key to use for the lock. Only other commands with the same key are blocked. Can use ${pwd} and ${cmd} placeholders. Defaults to current directory."
    )]
    //TODO @mverleg: impl
    pub lock_key: String,
    #[structopt(parse(try_from_str = parse_dur), short = 't', long = "timeout", default_value = "15 min", help = "Duration after which the waiting stops and the command fails. E.g. \"30 min\" or \"1 day -1 hour\".")]
    //TODO @mverleg: impl
    pub timeout: Duration,
    #[structopt(
    short = 'p',
    long = "progress",
    help = "Show an indicator that we are still waiting, what is running, and how frequently we are checking."
    )]
    //TODO @mverleg: impl
    pub show_progress: bool,
    #[structopt(
    long = "progress",
    help = "Show an indicator that we are still waiting, what is running, and how frequently we are checking."
    )]
    //TODO @mverleg: impl
    pub unlock: bool,
    #[structopt(subcommand)]
    //TODO @mverleg: impl
    pub cmd: CommandArgs,

    #[structopt(parse(try_from_str = parse_dur), short = 'd', long = "duration", default_value = "15 min", help = "Duration the cache should be valid for, e.g. \"30 min\" or \"1 day -1 hour\".")]
    pub duration: Duration,
    #[structopt(
    short = 'k',
    long = "key",
    default_value = "${pwd}_${cmd}.cache",
    help = "The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path."
    )]
    pub key: String,
    #[structopt(
    short = 'v',
    long,
    help = "Print extra information, e.g. whether the command was run or not"
    )]
    pub verbose: bool,

}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    DirLockedArgs::into_app().debug_assert()
}
