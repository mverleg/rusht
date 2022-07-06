use regex::Regex;

pub fn get_matches(pattern: &Regex, text: &str, mut handle_match: impl FnMut(String), first_only: bool, keep_unmatched: bool) {
    match pattern.captures(&text) {
        Some(captures) => {
            let mut caps = captures.iter();
            let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
            let mut any_groups = false;
            for mtch_opt in caps {
                if let Some(mtch) = mtch_opt {
                    handle_match(mtch.as_str().to_owned());
                }
                any_groups = true;
                if first_only {
                    break;
                }
            }
            if !any_groups {
                handle_match(full_match);
            }
        }
        None => {
            if keep_unmatched {
                handle_match(text)
            }
        }
    }
}

