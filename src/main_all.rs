use ::env_logger;

use ::clap::StructOpt;
use ::clap::Subcommand;
use ::rusht::cached::CachedArgs;
use rusht::cmd::{AddArgs, DoArgs, DropArgs, ListArgs};
use rusht::filter::{GrabArgs, UniqueArgs};
use rusht::find::DirWithArgs;
use rusht::wait::LockedArgs;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "rusht",
    about = "Single executable with all rusht utilities as subcommands."
)]
pub struct RushtArgs {
    #[clap(subcommand)]
    subcommand: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    Cached(CachedArgs),
    CmAdd(AddArgs),
    CmDo(DoArgs),
    CmList(ListArgs),
    CmDrop(DropArgs),
    DirWith(DirWithArgs),
    Grab(GrabArgs),
    Locked(LockedArgs),
    Unique(UniqueArgs),
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    CachedArgs::into_app().debug_assert()
}


fn main() {
    env_logger::init();

}
