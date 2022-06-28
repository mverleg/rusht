use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::find::{DirWithArgs, find_dir_with, PathModification};
use ::rusht::find::handle_dir_with;

fn main() {
    env_logger::init();
    let args = DirWithArgs::from_args();
    handle_dir_with(args)
}
