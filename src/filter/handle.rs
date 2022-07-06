use ::std::io::{BufRead, BufReader, stdin};
use ::std::process::exit;
use crate::common::{StdinReader, StdoutWriter};

use crate::filter::unique;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

pub fn handle_grab(args: GrabArgs) {
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    match grab(args, &mut StdinReader::new(), &mut StdoutWriter::new()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

pub fn handle_unique(args: UniqueArgs) {
    let mut lines = BufReader::new(stdin().lock()).lines();
    unique(args, || lines.next(), |line| println!("{}", line));
}
