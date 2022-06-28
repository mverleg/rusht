use ::clap::StructOpt;
use ::clap::Subcommand;
use ::env_logger;

use ::rusht::cached::CachedArgs;
use ::rusht::cached::handle_cached;
use ::rusht::cmd::{AddArgs, DoArgs, DropArgs, ListArgs};
use ::rusht::cmd::{handle_add, handle_do, handle_drop, handle_list};
use ::rusht::filter::{GrabArgs, UniqueArgs};
use ::rusht::filter::{handle_grab, handle_unique};
use ::rusht::find::DirWithArgs;
use ::rusht::find::handle_dir_with;
use ::rusht::wait::LockedArgs;
use ::rusht::wait::handle_locked;

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
    Cmadd(AddArgs),
    Cmdo(DoArgs),
    Cmlist(ListArgs),
    Cmdrop(DropArgs),
    //TODO @mverleg: get this to use dir_with
    DirWith(DirWithArgs),
    Grab(GrabArgs),
    Unique(UniqueArgs),
    Locked(LockedArgs),
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    RushtArgs::into_app().debug_assert()
}


fn main() {
    env_logger::init();
    let args = RushtArgs::from_args();
    match args.subcommand {
        SubCmd::Cached(sub_args) => handle_cached(sub_args),
        SubCmd::Cmadd(sub_args) => handle_add(sub_args),
        SubCmd::Cmadd(sub_args) => handle_add(sub_args),
        SubCmd::Cmdo(sub_args) => handle_do(sub_args),
        SubCmd::Cmlist(sub_args) => handle_list(sub_args),
        SubCmd::Cmdrop(sub_args) => handle_drop(sub_args),
        SubCmd::DirWith(sub_args) => handle_dir_with(sub_args),
        SubCmd::Grab(sub_args) => handle_grab(sub_args),
        SubCmd::Unique(sub_args) => handle_unique(sub_args),
        SubCmd::Locked(sub_args) => handle_locked(sub_args),
    }
}
