use ::clap::StructOpt;

use ::rusht::filter::GrabArgs;
use ::rusht::filter::handle_grab;

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    handle_grab(args)
}
