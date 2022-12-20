use ::std::time::Duration;

use ::clap::Parser;
use ::parse_duration0::parse as parse_dur;

use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired. Stderr is only shown on first run."
)]
pub struct CachedArgs {
    /// Duration the cache should be valid for, e.g. "30 min" or "1 day -1 hour".
    #[arg(value_parser = parse_dur, short = 'd', long = "duration", default_value = "15 min")]
    pub duration: Duration,
    #[arg(
        short = 'k',
        long = "key",
        default_value = "%{pwd}_%{env}_%{cmd}.cache",
        help = "The key to use for the cache. Can use %{pwd}, %{env} and %{cmd} placeholders. See long --help for more.",
        long_help = "The key to use for the cache. Can use %{pwd}, %{env} and %{cmd} placeholders.{n}{n}* %{git_uncommitted} contains a hash of the git index and unstaged files.{n}* %{git_head} contains the hash of the git head commit.{n}* %{git} is the combination of all git state.{n}* %{env} only contains non-inherited env."
    )]
    pub key: String,
    // /// Cache based on git state. If the head, index and unstaged changes are the exact same.
    // ///
    // /// This is just a short way to set --duration to a long time and --key to '%{git_head}_%{git_uncommitted}.cache'
    // #[structopt(short = 'g', long = "git", conflicts_with = "duration", conflicts_with = "key")]
    // pub git: bool,
    /// Print extra information, e.g. whether the command was run or not.
    /// When loading from cache, do not show the previous output.
    #[arg(short = 's', long)]
    pub no_cached_output: bool,
    #[arg(short = 'v', long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    CachedArgs::try_parse_from(&["cmd", "-v", "--", "ls"]).unwrap();
}
