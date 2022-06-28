use ::clap::StructOpt;
use ::clap::Subcommand;
use ::env_logger;

use ::rusht::cached::cached;
use ::rusht::cached::CachedArgs;
use ::rusht::cmd::{add_cmd, AddArgs, do_cmd, DoArgs, drop_cmd, DropArgs, list_cmds, ListArgs};
use ::rusht::filter::{grab, GrabArgs, unique, UniqueArgs};
use ::rusht::find::{DirWithArgs, find_dir_with};
use ::rusht::wait::{locked, LockedArgs};
use rusht::cached::handle_cached;
use rusht::cmd::{handle_add, handle_do, handle_drop, handle_list};

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
    RushtArgs::into_app().debug_assert()
}


fn main() {
    env_logger::init();
    let args = RushtArgs::from_args();
    match args.subcommand {
        SubCmd::Cached(sub_args) => handle_cached(sub_args),
        SubCmd::CmAdd(sub_args) => handle_add(sub_args),
        SubCmd::CmDo(sub_args) => handle_do(sub_args),
        SubCmd::CmList(sub_args) => handle_list(sub_args),
        SubCmd::CmDrop(sub_args) => handle_drop(sub_args),
        SubCmd::DirWith(sub_args) => todo!(),  // find_dir_with(sub_args),
        SubCmd::Grab(sub_args) => todo!(),  // grab(sub_args),
        SubCmd::Locked(sub_args) => todo!(),  // locked(sub_args),
        SubCmd::Unique(sub_args) => todo!(),  // unique(sub_args),
    }
    dbg!(args);
}
