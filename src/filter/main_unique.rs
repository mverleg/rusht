use ::clap::StructOpt;
use ::ustr::Ustr;

use ::rusht::common::{stdin_lines, EmptyLineHandling};
use ::rusht::filter::{unique, unique_prefix, UniqueArgs};

fn main() {
    env_logger::init();
    let args = UniqueArgs::from_args();
    let lines = stdin_lines(EmptyLineHandling::Drop)
        .iter()
        .map(|line| Ustr::from(line))
        .collect();
    let result = if args.prefix {
        unique_prefix(lines, args.order, args.keep)
    } else {
        unique(lines, args.order, args.keep)
    };
    for line in result {
        println!("{}", line);
    }
}
