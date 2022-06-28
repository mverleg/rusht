use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::filter::{unique, unique_prefix, UniqueArgs};
use ::rusht::filter::{grab, GrabArgs};
use ::rusht::filter::handle_grab;

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    handle_grab(args)
}
