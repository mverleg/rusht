use ::std::io;

use ::clap::StructOpt;
use ::regex::Regex;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grab",
    about = "Filter lines by regular expression, keeping only the matching capture group."
)]
pub struct GrabArgs {
    #[structopt(
        help = "Regular expression to match. Returns the capture group if any, or the whole match otherwise."
    )]
    pub pattern: Regex,
    #[structopt(
        short = '1',
        long,
        help = "Only print the first capture group, even if there are multiple"
    )]
    pub first_only: bool,
    #[structopt(
        short = 'k',
        long,
        help = "Keep the full line if it does not match the pattern"
    )]
    pub keep_unmatched: bool,
}

impl Default for GrabArgs {
    fn default() -> Self {
        GrabArgs {
            pattern: Regex::new(".*").unwrap(),
            first_only: false,
            keep_unmatched: false,
        }
    }
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    GrabArgs::into_app().debug_assert()
}

pub fn grab(
    args: GrabArgs,
    mut line_supplier: impl FnMut() -> Option<io::Result<String>>,
    mut consume: impl FnMut(String),
) -> Result<(), String> {
    while let Some(line_res) = line_supplier() {
        let line = match line_res {
            Ok(line) => line,
            Err(err) => return Err(format!("failed to read line: {}", err)),
        };
        match args.pattern.captures(&line) {
            Some(captures) => {
                let mut caps = captures.iter();
                let full_match = caps.next().unwrap().unwrap().as_str().to_owned();
                let mut any_groups = false;
                for mtch_opt in caps {
                    if let Some(mtch) = mtch_opt {
                        consume(mtch.as_str().to_owned());
                    }
                    any_groups = true;
                    if args.first_only {
                        break;
                    }
                }
                if !any_groups {
                    consume(full_match);
                }
            }
            None => {
                if args.keep_unmatched {
                    consume(line)
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_grab<S: Into<String>>(input: Vec<S>) -> Vec<String> {
        run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b").unwrap(),
                ..GrabArgs::default()
            },
            input,
        )
    }

    fn run_grab_arg<S: Into<String>>(args: GrabArgs, input: Vec<S>) -> Vec<String> {
        let mut res = vec![];
        let mut lines = input.into_iter().map(|v| Ok(v.into()));
        grab(args, || lines.next(), |line| res.push(line)).unwrap();
        res
    }

    #[test]
    fn no_lines() {
        let empty: Vec<String> = vec![];
        let res = run_grab(empty.clone());
        assert_eq!(res, empty);
    }
    #[test]
    fn empty_line() {
        let res = run_grab(vec![""]);
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[test]
    fn ignore_not_matching() {
        let res = run_grab(vec!["c"]);
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[test]
    fn ignore_only_group_matches() {
        let res = run_grab(vec!["aa"]);
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[test]
    fn match_single_group() {
        let res = run_grab(vec!["aab"]);
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn match_some_lines() {
        let res = run_grab(vec!["aab", "", "cab", "AAB"]);
        let expected: Vec<String> = vec!["aa".to_owned(), "a".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn first_of_multi_per_line() {
        let res = run_grab(vec!["aabab"]);
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn full_match_if_no_group() {
        let input = vec!["aab"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("a+b").unwrap(),
                ..GrabArgs::default()
            },
            input,
        );
        let expected: Vec<String> = vec!["aab".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn match_multiple_groups() {
        let input = vec!["aabccd"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                ..GrabArgs::default()
            },
            input,
        );
        let expected: Vec<String> = vec!["aa".to_owned(), "cc".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn match_only_first_groups() {
        let input = vec!["aabccd"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                first_only: true,
                ..GrabArgs::default()
            },
            input,
        );
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[test]
    fn keep_unmatched_lines() {
        let input = vec!["aabccd", "abc", "bcc"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                keep_unmatched: true,
                ..GrabArgs::default()
            },
            input,
        );
        let expected: Vec<String> = vec![
            "aa".to_owned(),
            "cc".to_owned(),
            "abc".to_owned(),
            "bcc".to_owned(),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn first_group_and_keep_unmatched() {
        let input = vec!["aabccd", "abc", "bcc"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                keep_unmatched: true,
                first_only: true,
                ..GrabArgs::default()
            },
            input,
        );
        let expected: Vec<String> = vec!["aa".to_owned(), "abc".to_owned(), "bcc".to_owned()];
        assert_eq!(res, expected);
    }
}
