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

#[derive(Debug)]
struct Span {
    start: NaiveTime,
    end: NaiveTime,
}

impl Span {
    fn dur_ms(&self) -> i64 {
        (self.end - self.start).num_milliseconds()
    }
}

fn run(args: &Args) -> Result<(), String> {
    let groups = find_logs(&args.file_pattern, &args.group_by);
    let cease_dur = groups.into_iter()
        .map(|(group, logs)|
            (group, logs.iter()
                .map(|log| matching_lines(log, &[args.line_pattern.clone()], &args.from, &args.to).pop().unwrap())
                .map(|lines| detect_spans(&lines, &args.line_pattern))
                .flat_map(|lines| lines.into_iter())
                .collect::<Vec<Span>>()))
        //.inspect(|(group, spans)| println!("{} has {} log-cease spans", group, spans.len()))
        .map(|(group, spans)| (group, spans.iter()
            .map(|span| span.dur_ms())
            .sum::<i64>()))
        .sorted_by(|left, right| left.0.cmp(&right.0))
        .for_each(|(group, dur)| println!("{}\t{} ms", group, dur));
    Ok(())
}

fn detect_spans(lines: &[String], re: &Regex) -> Vec<Span> {
    let mut spans = vec![];
    let mut prev_cease = None;
    for line in lines {
        if !re.is_match(line) {
            continue;
        }
        if let Some(time) = line_time(line) {
            if line.ends_with(" (ceased)") {
                if let Some(prev) = prev_cease {
                    eprintln!("encountered two ceased lines without a logged line");
                    continue;
                }
                prev_cease = Some(time)
            } else if line.ends_with(" (logged)") {
                //TODO: should also count spans that started before open
                if let Some(prev) = prev_cease {
                    let span = Span {
                        start: prev,
                        end: time,
                    };
                    spans.push(span);
                    prev_cease = None
                } else {
                    continue;
                }
            }
        }
    }
    spans
}
