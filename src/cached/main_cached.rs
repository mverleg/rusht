use ::std::env::current_dir;

use ::structopt::StructOpt;

use ::rusht::cached::cached;
use ::rusht::cached::CachedArgs;
use ::rusht::common::Task;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    match cached(args) {

    }
}
