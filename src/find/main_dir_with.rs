use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::find::{find_dir_with, DirWithArgs, PathModification};

fn main() {
    env_logger::init();
    let args = DirWithArgs::from_args();
    handle_dir_with(args)
}
