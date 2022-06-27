use ::std::io::{BufRead, BufReader, stdin};

use ::clap::StructOpt;
use ::ustr::Ustr;

use ::rusht::common::{EmptyLineHandling, stdin_lines};
use ::rusht::filter::{unique, unique_prefix, UniqueArgs};
use ::rusht::filter::{grab, GrabArgs};

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    let mut reader = BufReader::new(stdin().lock());
    let line_supplier = || reader.lines().next();
    grab(args, |line| println!("{}", line), line_supplier);
}
