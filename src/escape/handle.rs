use crate::ExitStatus;
use ::std::io::{stdin, BufRead, BufReader};
use log::debug;
use crate::common::{StdinReader, VecReader};

use super::namesafe;
use super::NamesafeArgs;

pub async fn handle_namesafe(args: NamesafeArgs) -> ExitStatus {
    let mut lines = BufReader::new(stdin().lock()).lines();
    let res = match args.input.clone() {
        Some(inp) => {
            assert!(!args.allow_empty);
            assert!(!args.single_line);
            debug!("namesafe getting input from provided string, ignoring stdin");
            namesafe(args, &mut VecReader::new(vec![inp]), |line| println!("{}", line)).await
        }
        None => {
            debug!("namesafe getting input from stdin; allow_empty={}, single_line={}", args.allow_empty, args.single_line);
            namesafe(args, &mut StdinReader::new(), |line| println!("{}", line)).await
        }
    };
    match res {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}
