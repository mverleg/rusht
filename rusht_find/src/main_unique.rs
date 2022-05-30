use ::std::io::BufRead;
use ::std::io::stdin;

use ::structopt::StructOpt;

use ::rusht_find::unique;
use ::rusht_find::unique_prefix;
use ::rusht_find::UniqueArgs;

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    let lines = stdin().lock().lines()
        .map(|line| line.expect("a line was not utf8"))
        .collect::<Vec<String>>();
    if args.prefix {
        unique_prefix(&lines, args.sorted);
    } else {
        unique(&lines, args.sorted, args.find_duplicates);
    }
}
