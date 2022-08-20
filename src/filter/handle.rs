use crate::common::{StdinReader, StdoutWriter};
use crate::filter::filter;
use crate::filter::unique;
use crate::filter::FilterArgs;
use crate::filter::UniqueArgs;
use crate::ExitStatus;

use super::{grab, GrabArgs};

pub async fn handle_grab(args: GrabArgs) -> ExitStatus {
    match grab(args, StdinReader::new(), StdoutWriter::new()).await {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}

pub async fn handle_unique(args: UniqueArgs) -> ExitStatus {
    unique(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
    ExitStatus::ok()
}

pub async fn handle_filter(args: FilterArgs) -> ExitStatus {
    filter(args, &mut StdinReader::new(), &mut StdoutWriter::new()).await;
    ExitStatus::ok()
}
