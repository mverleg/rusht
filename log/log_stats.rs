#![allow(unused)]

use ::chrono::NaiveTime;
use ::chrono::ParseResult;
use ::itertools::Itertools;
use ::regex::Regex;
use ::std::cmp::Ordering;
use ::std::collections::HashMap;
use ::std::fs::read_to_string;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::exit;
use ::std::str::FromStr;
use ::structopt::StructOpt;
use ::strum_macros::EnumString;
use ::walkdir::WalkDir;

use crate::log_operations::Operation;

mod log_operations;

#[allow(non_camel_case_types)]
#[derive(Debug, EnumString)]
enum Sort {
    group,
    count,
}

impl Sort {
    fn cmp(&self, left_group: &str, left_count: i64, right_group: &str, right_count: i64) -> Ordering {
        match self {
            Sort::group => left_group.cmp(right_group),
            Sort::count => right_count.cmp(&left_count),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "logstat", about = "extract metrics from imc log files")]
struct Args {
    #[structopt(required = true, short = "c", long, help = "Patterns that the lines should match")]
    count_patterns: Vec<Regex>,
    #[structopt(required = true, short = "s", long, help = "Pattern that the ceasable lines should match")]
    span_pattern: Regex,
    #[structopt(long, short = "f", default_value = "\\.(log|txt)$", help = "Regex to limit paths that are considered logs")]
    file_pattern: Regex,
    #[structopt(long, help = "Regex for log path, containing a single capture group that will be used to group logs, summing counts within the group")]
    group_by: Option<Regex>,
    #[structopt(long, help = "Time (including seconds) from which to analyze")]
    from: Option<NaiveTime>,
    #[structopt(long, help = "Time (including seconds) upto which to analyze")]
    to: Option<NaiveTime>,
    #[structopt(long, help = "What property to sort results by", default_value = "Group")]
    order_by: Sort,
}

fn main() {
    let args = Args::from_args();
    args.count_patterns.into_iter()
        .map(|count_re| Operation::RegexCount { regex: count_re })
    args.span_pattern.into_iter()
        .map(|span_re| Operation::RegexCeaseSpan { regex: span_re })
    run(&args).unwrap();
}
