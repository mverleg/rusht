use ::structopt::StructOpt;

use ::rusht::cached::CachedArgs;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    dbg!(&args);  //TODO @mark:
    //TODO @mark:
}
