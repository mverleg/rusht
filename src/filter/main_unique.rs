use ::clap::StructOpt;

use ::rusht::filter::UniqueArgs;
use ::rusht::filter::handle_unique;

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    handle_unique(args)
}
