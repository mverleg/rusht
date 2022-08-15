use ::clap::StructOpt;
use ::clap::Subcommand;
use ::env_logger;

use ::rusht::cached::handle_cached;
use ::rusht::cached::CachedArgs;
use ::rusht::cmd::{handle_add, handle_do, handle_drop, handle_list};
use ::rusht::cmd::{AddArgs, DoArgs, DropArgs, ListArgs};
use ::rusht::escape::NamesafeArgs;
use ::rusht::filter::{handle_grab, handle_unique};
use ::rusht::filter::{GrabArgs, UniqueArgs};
use ::rusht::find::handle_dir_with;
use ::rusht::find::DirWithArgs;
use ::rusht::wait::handle_locked;
use ::rusht::wait::LockedArgs;
use rusht::escape::handle_namesafe;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "mvnw",
    about = "Wrapper for maven (daemon) to add speed flags. Needs maven and git."
)]
pub struct MvnwArgs {
    #[structopt(short = 'c', long, help = "Do a clean build and update snapshots")]
    pub clean: bool,
    #[structopt(
        short = 'x',
        long = "hash",
        default_value = "changed",  //TODO @mverleg: not sure why Default impl doesn't work
        help = "In which cases to include a hash in the name ([a]lways, [c]hanged, too-[l]ong, [n]ever)."
    )]
    pub hash_policy: BuildPolicy,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BuildPolicy {
    commit/uncommitted/branch/all/recent
    Commit,
    #[default]
    Uncommitted,
    Branch,
    Recent,
    All,
}

impl FromStr for BuildPolicy {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(match text.to_lowercase().as_str() {
            "always" | "a" => HashPolicy::Always,
            "changed" | "c" => HashPolicy::Changed,
            "too-long" | "long" | "l" => HashPolicy::TooLong,
            "never" | "n" => HashPolicy::TooLong,
            other => return Err(format!("unknown hash policy: {}", other)),
        })
    }
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    MvnwArgs::into_app().debug_assert()
}
