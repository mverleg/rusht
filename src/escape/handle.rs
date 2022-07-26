use ::std::io::{stdin, BufRead, BufReader};
use ::std::process::exit;

use super::namesafe;
use super::NamesafeArgs;

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
