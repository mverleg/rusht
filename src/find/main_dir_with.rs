use ::clap::StructOpt;

use ::rusht::find::handle_dir_with;
use ::rusht::find::DirWithArgs;

fn main() {
    env_logger::init();
    let args = DirWithArgs::from_args();
    handle_dir_with(args)
}
