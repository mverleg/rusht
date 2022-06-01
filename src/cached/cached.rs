use ::std::fs;
use ::std::path::PathBuf;
use ::std::time::Duration;

use ::itertools::Itertools;
use ::regex::Regex;
use ::structopt::StructOpt;
use ::ustr::Ustr;

use crate::find::unique_prefix;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cached",
    about = "Cache the output of a command for a given duration, running it only if there is no cache or it has expired."
)]
pub struct CachedArgs {
    #[structopt(parse(try_from_str = parse_duration), short = "d", long = "duration", default_value = "15 min", help = "Duration the cache should be valid for, e.g. \"30 min\" or \"1 day -1 hour\".")]
    pub duration: Duration,
    #[structopt(short = "k", long = "key", default_value = "${pwd}_${cmd}.cache", help = "The key to use for the cache. Can use ${pwd} and ${cmd} placeholders.")]
    pub key: Duration,
    #[structopt(subcommand)]
    pub cmd: CachedArgsExtra,
}

#[derive(Debug, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum CachedArgsExtra {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

fn parse_duration(txt: &str) -> Result<Duraton, String> {
    unimplemented!();
}

