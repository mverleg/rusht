use ::clap::StructOpt;
use ::regex::Regex;

use crate::common::{get_matches, LineReader, LineWriter};

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

#[async_std::test]
async fn test_cli_args() {
    use clap::IntoApp;
    GrabArgs::into_app().debug_assert()
}

pub async fn grab(
    args: GrabArgs,
    mut reader: impl LineReader,
    mut writer: impl LineWriter,
) -> Result<(), String> {
    while let Some(line) = reader.read_line().await {
        get_matches(
            &args.pattern,
            line,
            &mut writer,
            args.first_only,
            args.keep_unmatched,
        )
        .await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::async_std;

    use crate::common::{VecReader, VecWriter};

    use super::*;

    async fn run_grab<S: Into<String>>(input: Vec<S>) -> Vec<String> {
        run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b").unwrap(),
                ..GrabArgs::default()
            },
            input,
        )
        .await
    }

    async fn run_grab_arg<S: Into<String>>(args: GrabArgs, input: Vec<S>) -> Vec<String> {
        let res = VecWriter::new();
        grab(args, VecReader::new(input), res.clone())
            .await
            .unwrap();
        res.get()
    }

    #[async_std::test]
    async fn no_lines() {
        let empty: Vec<String> = vec![];
        let res = run_grab(empty.clone()).await;
        assert_eq!(res, empty);
    }

    #[async_std::test]
    async fn empty_line() {
        let res = run_grab(vec![""]).await;
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn ignore_not_matching() {
        let res = run_grab(vec!["c"]).await;
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn ignore_only_group_matches() {
        let res = run_grab(vec!["aa"]).await;
        let expected: Vec<String> = vec![];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn match_single_group() {
        let res = run_grab(vec!["aab"]).await;
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn match_some_lines() {
        let res = run_grab(vec!["aab", "", "cab", "AAB"]).await;
        let expected: Vec<String> = vec!["aa".to_owned(), "a".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn first_of_multi_per_line() {
        let res = run_grab(vec!["aabab"]).await;
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn full_match_if_no_group() {
        let input = vec!["aab"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("a+b").unwrap(),
                ..GrabArgs::default()
            },
            input,
        )
        .await;
        let expected: Vec<String> = vec!["aab".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn match_multiple_groups() {
        let input = vec!["aabccd"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                ..GrabArgs::default()
            },
            input,
        )
        .await;
        let expected: Vec<String> = vec!["aa".to_owned(), "cc".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn match_only_first_groups() {
        let input = vec!["aabccd"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                first_only: true,
                ..GrabArgs::default()
            },
            input,
        )
        .await;
        let expected: Vec<String> = vec!["aa".to_owned()];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn keep_unmatched_lines() {
        let input = vec!["aabccd", "abc", "bcc"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                keep_unmatched: true,
                ..GrabArgs::default()
            },
            input,
        )
        .await;
        let expected: Vec<String> = vec![
            "aa".to_owned(),
            "cc".to_owned(),
            "abc".to_owned(),
            "bcc".to_owned(),
        ];
        assert_eq!(res, expected);
    }

    #[async_std::test]
    async fn first_group_and_keep_unmatched() {
        let input = vec!["aabccd", "abc", "bcc"];
        let res = run_grab_arg(
            GrabArgs {
                pattern: Regex::new("(a+)b(c{2})").unwrap(),
                keep_unmatched: true,
                first_only: true,
                ..GrabArgs::default()
            },
            input,
        )
        .await;
        let expected: Vec<String> = vec!["aa".to_owned(), "abc".to_owned(), "bcc".to_owned()];
        assert_eq!(res, expected);
    }
}
