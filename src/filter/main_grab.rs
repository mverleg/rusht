use ::std::io::{BufRead, BufReader, stdin};

use ::clap::StructOpt;
use ::ustr::Ustr;

use ::rusht::common::{EmptyLineHandling, stdin_lines};
use ::rusht::filter::{unique, unique_prefix, UniqueArgs};
use ::rusht::filter::{grab, GrabArgs};

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    grab(args, line_supplier, |line| println!("{}", line));
}
