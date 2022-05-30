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

use crate::log_util::*;

mod log_util;

fn run(args: &Args) -> Result<(), String> {
    let groups = find_logs(&args.file_pattern, &args.group_by);
    let counts = groups.into_iter()
        .map(|(group, logs)| (group, logs.into_iter()
            .map(|log| count_lines(&log, &args.line_patterns, &args.from, &args.to))
            .reduce(|a, b| a.iter().zip(b.iter()).map(|(&x, &y)| x + y).collect())
            .unwrap()))
        .collect::<Vec<(String, Vec<i64>)>>()
        .into_iter()
        .sorted_by(|left, right| args.order_by.cmp(&left.0, left.1[0], &right.0, right.1[0]))
        .collect::<Vec<_>>();
    println!("\t'{}'", args.line_patterns.iter().join("'\t'"));
    counts.iter()
        .for_each(|(group, count)| println!("{}\t{}", group, count.iter().join("\t")));
    Ok(())
}

pub fn count_lines(path: &Path, line_regexs: &[Regex], from: &Option<NaiveTime>, to: &Option<NaiveTime>) -> Vec<i64> {
    matching_lines(path, line_regexs, from, to).iter()
        .map(|lines| lines.len() as i64)
        .collect::<Vec<_>>()
}
