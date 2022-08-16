use ::std::time::Duration;

use ::clap::StructOpt;
use ::parse_duration0::parse as parse_dur;

use crate::common::CommandArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired. Stderr is only shown on first run."
)]
pub struct CachedArgs {
    #[structopt(parse(try_from_str = parse_dur), short = 'd', long = "duration", default_value = "15 min")]
    /// Duration the cache should be valid for, e.g. "30 min" or "1 day -1 hour".
    pub duration: Duration,
    #[structopt(
        short = 'k',
        long = "key",
        default_value = "${pwd}_${env}_${cmd}.cache"
    )]
    /// The key to use for the cache. Can use ${pwd}, ${env} and ${cmd} placeholders (${env} only contains non-inherited env).
    pub key: String,
    #[structopt(short = 'v', long)]
    /// Print extra information, e.g. whether the command was run or not.
    pub verbose: bool,
    #[structopt(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    CachedArgs::into_app().debug_assert()
}
