use ::std::io::{stdin, BufRead, BufReader};
use ::std::process::exit;

use ::ustr::Ustr;

use crate::common::EmptyLineHandling;

use super::{grab, GrabArgs};
use super::{unique, unique_prefix, UniqueArgs};

pub fn handle_grab(args: GrabArgs) {
    let mut lines = BufReader::new(stdin().lock()).lines();
    let line_supplier = || lines.next();
    match grab(args, line_supplier, |line| println!("{}", line)) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

pub fn handle_unique(args: UniqueArgs) {
    let lines = crate::common::stdin_lines(EmptyLineHandling::Drop)
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
