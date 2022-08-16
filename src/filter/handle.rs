use ::std::process::exit;

use crate::common::{StdinReader, StdoutWriter};
use crate::filter::filter;
use crate::filter::unique;
use crate::filter::FilterArgs;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

pub async fn handle_grab(args: GrabArgs) {
    match grab(args, StdinReader::new(), StdoutWriter::new()).await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

pub async fn handle_unique(args: UniqueArgs) {
    unique(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
}

pub async fn handle_filter(args: FilterArgs) {
    filter(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
}
