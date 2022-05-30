use ::std::io::BufRead;
use ::std::io::stdin;

use ::structopt::StructOpt;

use ::rusht_find::unique;
use ::rusht_find::UniqueArgs;

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    let lines = stdin().lock().lines().collect();
    if args.dup {}
    unique(args, lines);
}
