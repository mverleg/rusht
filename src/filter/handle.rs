use ::log::debug;

use crate::common::{DiscardWriter, RejectStdin, StdinReader, StdWriter, VecReader};
use crate::ExitStatus;
use crate::filter::BetweenArgs;
use crate::filter::between;
use crate::filter::filter;
use crate::filter::FilterArgs;
use crate::filter::unique;
use crate::filter::UniqueArgs;

use super::{grab, GrabArgs};

//TODO @mverleg: too much code in handle, should be inside grab?
pub async fn handle_grab(args: GrabArgs) -> ExitStatus {
    let quiet = args.quiet;
    let expect_match = args.expect_match;
    let expect_no_match = args.expect_no_match;
    let pattern_str = args.pattern.as_str().to_owned();
    assert!(!(expect_match && expect_no_match), "cannot combine -expect-match and --expect-no-match");
    if quiet {
        assert!(expect_match || expect_no_match, "grab: --quiet only usable when --expect-match or --expect-no-match");
    }
    let grab_res = match (args.input.clone(), quiet) {
        (Some(inp), true) => {
            debug!("grab getting input from provided string, discarding output");
            grab(args, VecReader::new(vec![inp]), DiscardWriter::new()).await
        },
        (Some(inp), false) => {
            debug!("grab getting input from provided string, printing output");
            grab(args, VecReader::new(vec![inp]), StdWriter::stdout()).await
        }
        (None, true) => {
            debug!("grab getting input from stdin, discarding output");
            grab(args, StdinReader::new(), DiscardWriter::new()).await
        }
        (None, false) => {
            debug!("grab getting input from stdin, printing output");
            grab(args, StdinReader::new(), StdWriter::stdout()).await
        }
    };
    match grab_res {
        Ok(match_cnt) => {
            exit_from_match(match_cnt, expect_match, expect_no_match, &pattern_str, quiet)
        },
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}

fn exit_from_match(match_cnt: u32, expect_match: bool, expect_no_match: bool, pattern: &str, quiet: bool) -> ExitStatus {
    if expect_match {
        return if match_cnt == 0 {
            debug!("grab failed because --expect-match but no results");
            if !quiet {
                eprintln!("grab expected result for '{pattern}' but did not find");
            }
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
            if !quiet {
                eprintln!("grab expected no result for '{pattern}' but found {match_cnt}");
            }
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

#[cfg(test)]
mod tests {
    use regex::Regex;
    use super::*;

    #[async_std::test]
    async fn grab_input() {
        let res = handle_grab(GrabArgs {
            pattern: Regex::new("bc").unwrap(),
            input: Some("abcd-abcd\nabcd".to_owned()),
            first_match_only: true,
            first_capture_only: true,
            keep_unmatched: false,
            max_lines: Some(1),
            expect_match: true,
            expect_no_match: false,
            quiet: true,
        }).await;
        assert!(res.is_ok())
    }
}
