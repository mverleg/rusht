use ::std::io::{BufRead, BufReader, stdin};
use ::std::process::exit;
use crate::common::{StdinReader, StdoutWriter};

use crate::filter::unique;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

pub async fn handle_grab(args: GrabArgs) {
    match grab(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await {
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
