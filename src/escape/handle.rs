use crate::ExitStatus;
use ::std::io::{stdin, BufRead, BufReader};

use super::namesafe;
use super::NamesafeArgs;

pub fn handle_namesafe(args: NamesafeArgs) -> ExitStatus {
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    match namesafe(args, line_supplier, |line| println!("{}", line)) {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}
