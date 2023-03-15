use crate::ExitStatus;
use ::std::io::{stdin, BufRead, BufReader};
use log::debug;
use crate::common::{StdinReader, StdWriter, VecReader};

use super::namesafe;
use super::NamesafeArgs;

pub async fn handle_namesafe(args: NamesafeArgs) -> ExitStatus {
    let res = match args.input.clone() {
        Some(inp) => {
            assert!(!args.allow_empty);
            assert!(!args.single_line);
            debug!("namesafe getting input from provided string, ignoring stdin");
            namesafe(args, &mut VecReader::new(vec![inp]), &mut StdWriter::stdout()).await
        }
        None => {
            debug!("namesafe getting input from stdin; allow_empty={}, single_line={}", args.allow_empty, args.single_line);
            namesafe(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await
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
