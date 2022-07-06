use ::std::io::{BufRead, BufReader, stdin};
use ::std::process::exit;

use ::ustr::Ustr;

use crate::common::EmptyLineHandling;
use crate::filter::{Order, unique_nosort};

use super::{grab, GrabArgs};
use super::{unique_prefix, UniqueArgs};

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
    //TODO @mverleg: move all this to a lib function (except the stdout)
    if args.prefix {
        unique_prefix(lines, args.order, args.keep).iter()
            .for_each(|line| println!("{}", line));
    } else {
        if Order::SortAscending == args.order {
            let mut matches = vec![];
            unique_nosort(lines, args.keep, &args.by, |line| matches.push(line));
            order_inplace(&mut matches);
            matches.iter().for_each(|line| println!("{}", line))
        } else {
            assert_eq!(Order::Preserve, args.order);
            unique_nosort(lines, args.keep, &args.by, |line| println!("{}", line))
        }
    };
}
