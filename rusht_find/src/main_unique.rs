use ::structopt::StructOpt;

use ::rusht_find::read_input_lines;
use ::rusht_find::unique;
use ::rusht_find::unique_prefix;
use ::rusht_find::UniqueArgs;

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    let lines = read_input_lines();
    let result = if args.prefix {
        unique_prefix(lines, args.order, args.keep)
    } else {
        unique(lines, args.order, args.keep)
    };
    for line in result {
        println!("{}", line);
    }
}
