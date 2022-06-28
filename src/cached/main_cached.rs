use ::std::process::exit;

use ::clap::StructOpt;
use ::env_logger;

use ::rusht::cached::cached;
use ::rusht::cached::CachedArgs;
use ::rusht::cached::CacheStatus;
use ::rusht::cached::handle_cached;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    handle_cached(args)
}
