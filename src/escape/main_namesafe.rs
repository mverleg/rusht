use ::clap::StructOpt;

use super::handle_namesafe;
use super::NamesafeArgs;

fn main() {
    env_logger::init();
    let args = NamesafeArgs::from_args();
    handle_namesafe(args)
}
