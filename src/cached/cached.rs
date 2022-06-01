use ::std::time::Duration;

use ::parse_duration::parse as parse_dur;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired."
)]
pub struct CachedArgs {
    #[structopt(parse(try_from_str = parse_dur), short = "d", long = "duration", default_value = "15 min", help = "Duration the cache should be valid for, e.g. \"30 min\" or \"1 day -1 hour\".")]
    pub duration: Duration,
    #[structopt(short = "k", long = "key", default_value = "${pwd}_${cmd}.cache", help = "The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path.")]
    pub key: String,
    #[structopt(subcommand)]
    pub cmd: CachedArgsExtra,
}

#[derive(Debug, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum CachedArgsExtra {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheString {
    RanSuccessfully,
    FromCache,
    Failed(u8),
}

fn cached(args: CachedArgs) -> Result<CacheString, String> {
    //TODO @mark: The key to use for the cache. Can use ${pwd} and ${cmd} placeholders. If it contains a / it will be considered a full path.


    unimplemented!()
}
