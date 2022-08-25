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
    dbg!(&pattern);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
    let mut any_matches = false;
    // Iterate over all the times the complete pattern matches
    for captures in pattern.captures(text) {
        any_matches = true;
        dbg!(&captures);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
        let mut caps = captures.iter();
        let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
        let mut any_groups = false;
        // Within a pattern match, iterate over the capture groups
        for mtch_opt in caps {
            dbg!(mtch_opt);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
            any_groups = true;
            if let Some(mtch) = mtch_opt {
                writer.write_line(mtch.as_str()).await;
                match_cnt += 1
            }
            if first_only {
                dbg!("first_only");  //TODO @mverleg: TEMPORARY! REMOVE THIS!
                break;
            }
        }
        dbg!(match_cnt);  //TODO @mverleg: TEMPORARY! REMOVE THIS!
        if !any_groups {
            dbg!("full match");  //TODO @mverleg: TEMPORARY! REMOVE THIS!
            writer.write_line(full_match).await;
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
