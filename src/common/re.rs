use crate::common::LineWriter;
use ::regex::Regex;

pub async fn get_matches(
    pattern: &Regex,
    text: &str,
    writer: &mut impl LineWriter,
    first_match_only: bool,
    first_capture_only: bool,
    keep_unmatched: bool,
) -> u32 {
    let mut match_cnt = 0;
    let mut any_matches = false;
    // Iterate over all the times the complete pattern matches
    for captures in pattern.captures_iter(text) {
        any_matches = true;
        let mut caps = captures.iter();
        let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
        let mut any_groups = false;
        // Within a pattern match, iterate over the capture groups
        for mtch_opt in caps {
            any_groups = true;
            if let Some(mtch) = mtch_opt {
                writer.write_line(mtch.as_str()).await;
                match_cnt += 1
            }
            if first_capture_only {
                break;
            }
        }
        if !any_groups {
            writer.write_line(full_match).await;
        }
        if first_match_only {
            break;
        }
    }
    if !any_matches && keep_unmatched {
        writer.write_line(text.to_owned()).await
    }
    match_cnt
}

pub fn get_first_match_or_all<'a>(pattern: &Option<Regex>, text: &'a str) -> &'a str {
    if let Some(re) = pattern {
        if let Some(captures) = re.captures(text) {
            let mut caps = captures.iter();
            let full_match = caps.next().unwrap().unwrap().as_str();
            if let Some(mtch) = caps.flatten().next() {
                return mtch.as_str();
            }
            return full_match;
        }
    }
    text
}
