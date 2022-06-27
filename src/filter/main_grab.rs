use ::std::io::{BufRead, BufReader, stdin};
use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::filter::{unique, unique_prefix, UniqueArgs};
use ::rusht::filter::{grab, GrabArgs};

fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    match grab(args, line_supplier, |line| println!("{}", line)) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
    }
}
