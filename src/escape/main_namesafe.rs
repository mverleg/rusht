use ::clap::Parser;

use ::rusht::escape::handle_namesafe;
use ::rusht::escape::NamesafeArgs;

fn main() {
    env_logger::init();
    let args = NamesafeArgs::from_args();
    handle_namesafe(args)
}
