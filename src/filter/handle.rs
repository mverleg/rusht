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
    let quiet = args.quiet;
    let expect_match = args.expect_match;
    let expect_no_match = args.expect_no_match;
    assert!(!(expect_match && expect_no_match), "cannot combine -expect-match and --expect-no-match");
    if quiet {
        assert!(expect_match || expect_no_match, "grab: --quiet only usable when --expect-match or --expect-no-match");
    }
    let grab_res = if quiet {
        grab(args, StdinReader::new(), DiscardWriter::new()).await
    } else {
        grab(args, StdinReader::new(), StdWriter::stdout()).await
    };
    match grab_res {
        Ok(match_cnt) => {
            exit_from_match(match_cnt, expect_match, expect_no_match)
        },
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}

fn exit_from_match(match_cnt: u32, expect_match: bool, expect_no_match: bool) -> ExitStatus {
    if expect_match {
        return if match_cnt == 0 {
            debug!("grab failed because --expect-match but no results");
            ExitStatus::err()
        } else {
            debug!("grab succeeded because --expect-match and {} results", match_cnt);
            ExitStatus::ok()
        }
    }
    if expect_no_match {
        return if match_cnt == 0 {
            debug!("grab succeeded because --expect-no-match with no results");
            ExitStatus::ok()

        } else {
            debug!("grab failed because --expect-no-match but {} results", match_cnt);
            ExitStatus::err()
        }
    }
    debug!("grab succeeded with {} results because no --expect-match or --expect-no-match", match_cnt);
    ExitStatus::ok()
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
