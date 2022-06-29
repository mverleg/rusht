use ::clap::StructOpt;

use ::rusht::filter::handle_grab;
use ::rusht::filter::GrabArgs;

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    handle_grab(args)
}
