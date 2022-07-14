use ::std::io::{BufRead, BufReader, stdin};
use ::std::process::exit;

use super::NamesafeArgs;
use super::namesafe;

pub fn handle_namesafe(args: NamesafeArgs) {
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    match namesafe(args, line_supplier, |line| println!("{}", line)) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
