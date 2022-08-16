use crate::common::LineWriter;
use ::regex::Regex;

pub async fn get_matches(
    pattern: &Regex,
    text: &str,
    writer: &mut impl LineWriter,
    first_only: bool,
    keep_unmatched: bool,
) -> u32 {
    let mut match_cnt = 0;
    match pattern.captures(text) {
        Some(captures) => {
            let mut caps = captures.iter();
            let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
            let mut any_groups = false;
            for mtch_opt in caps {
                any_groups = true;
                if let Some(mtch) = mtch_opt {
                    writer.write_line(mtch.as_str()).await;
                    match_cnt += 1
                }
                if first_only {
                    break;
                }
            }
            if !any_groups {
                writer.write_line(full_match).await;
            }
        }
        None => {
            if keep_unmatched {
                writer.write_line(text.to_owned()).await
            }
        }
    }
    match_cnt
}

pub fn get_first_match_or_all<'a>(pattern: &Option<Regex>, text: &'a str) -> &'a str {
    if let Some(re) = pattern {
        if let Some(captures) = re.captures(text) {
            let mut caps = captures.iter();
            let full_match = caps.next().unwrap().unwrap().as_str();
            for mtch_opt in caps {
                if let Some(mtch) = mtch_opt {
                    return mtch.as_str();
                }
            }
            return full_match;
        }
    }
    text
}
