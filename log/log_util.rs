use ::chrono::NaiveTime;
use ::chrono::ParseResult;
use ::itertools::Itertools;
use ::regex::internal::Input;
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

pub fn find_logs(pattern: &Regex, group_by: &Option<Regex>) -> HashMap<String, Vec<PathBuf>> {
    let logs = WalkDir::new(".").into_iter()
        .filter_map(|e| e.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| pattern.is_match(path.to_str().unwrap()))
        //.inspect(|p| println!("{}", p.to_string_lossy()))
        .into_group_map_by(|path| match group_by {
            Some(re) => re.captures(path.to_str().unwrap()).expect("no regex match for log file path grouping")
                .get(1).expect("group pattern did not have regex capture group?").as_str().to_owned(),
            None => path.to_str().unwrap().to_owned(),
        });
    if logs.is_empty() {
        eprintln!("no logs matched pattern '{}'", pattern.as_str());
        exit(1)
    }
    let (group, items) = logs.iter().next().unwrap();
    eprintln!("found {} logs in {} group(s), e.g. '{}' in '{}'",
              logs.iter().map(|(_, grp)| grp.len()).sum::<usize>(),
              logs.len(), items[0].to_string_lossy(), group);
    logs
}

pub fn matching_lines(path: &Path, line_regexs: &[Regex], from: &Option<NaiveTime>, to: &Option<NaiveTime>) -> Vec<Vec<String>> {
    let content = read_to_string(path).unwrap();
    let timed_lines = content.lines()
        .filter(|line| is_time_in_range(line, from, to))
        .collect::<Vec<_>>();
    line_regexs.iter()
        .map(|re| timed_lines.iter()
            .filter(|line| re.is_match(line))
            .map(|line| line.to_string())
            .collect::<Vec<String>>())
        .collect::<Vec<_>>()
}

pub fn line_time(line: &str) -> Option<NaiveTime> {
    if let Some((head, tail)) = line.split_once(" ") {
        match NaiveTime::parse_from_str(head, "%H:%M:%S%.f") {
            Ok(time) => Some(time),
            Err(err) => {
                if line[0..2].parse::<u8>().is_ok() {
                    eprintln!("failed to parse time, error {} in '{}'", err, line);
                }
                None
            },
        }
    } else {
        None
    }
}

pub fn is_time_in_range(line: &str, from: &Option<NaiveTime>, to: &Option<NaiveTime>) -> bool {
    if from.is_none() && to.is_none() {
        return true;
    }
    if let Some(time) = line_time(line) {
        if let Some(from) = from {
            if time < *from {
                return false
            }
        }
        if let Some(to) = to {
            if time > *to {
                return false
            }
        }
    }
    true
}
