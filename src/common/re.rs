use ::regex::Regex;
use crate::common::LineWriter;

pub async fn get_matches(pattern: &Regex, text: &str, writer: &mut impl LineWriter, first_only: bool, keep_unmatched: bool) {
    match pattern.captures(&text) {
        Some(captures) => {
            let mut caps = captures.iter();
            let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
            let mut any_groups = false;
            for mtch_opt in caps {
                if let Some(mtch) = mtch_opt {
                    writer.write_line(mtch.as_str()).await;
                }
                any_groups = true;
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
}
