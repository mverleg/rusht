use ::log::debug;

use crate::common::{DiscardWriter, StdinReader, StdWriter};
use crate::ExitStatus;
use crate::filter::BetweenArgs;
use crate::filter::between;
use crate::filter::filter;
use crate::filter::FilterArgs;
use crate::filter::unique;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

pub async fn handle_grab(args: GrabArgs) -> ExitStatus {
    let quiet = args.exit_code;
    let exit_code = args.exit_code;
    let grab_res = if quiet {
        grab(args, StdinReader::new(), DiscardWriter::new()).await
    } else {
        grab(args, StdinReader::new(), StdWriter::stdout()).await
    };
    match grab_res {
        Ok(match_cnt) => {
            if exit_code && match_cnt == 0 {
                debug!("grab failing because of no results and --exit-code was requested");
                ExitStatus::err()
            } else {
                ExitStatus::ok()
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}

pub async fn handle_unique(args: UniqueArgs) -> ExitStatus {
    unique(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await;
    ExitStatus::ok()
}

pub async fn handle_filter(args: FilterArgs) -> ExitStatus {
    filter(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await;
    ExitStatus::ok()
}

pub async fn handle_between(args: BetweenArgs) -> ExitStatus {
    match between(args, &mut StdinReader::new(), &mut StdWriter::stdout()).await {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}
