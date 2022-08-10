use crate::common::{FirstItemWriter, LineWriter};
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

pub async fn get_first_match_or_all(pattern: &Option<Regex>, line: &str) -> String {
    if let Some(re) = pattern {
        let mut first_writer = FirstItemWriter::new();
        get_matches(re, line, &mut first_writer, true, true).await;
        if let Some(val) = first_writer.get() {
            return val
        }
    }
    line.to_owned()
}
