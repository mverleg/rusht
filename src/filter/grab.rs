use ::log::debug;

use crate::common::{get_matches, LineReader, LineWriter};
use crate::filter::GrabArgs;

pub async fn grab(
    args: GrabArgs,
    mut reader: impl LineReader,
    mut writer: impl LineWriter,
) -> Result<u32, String> {
    if let Some(max) = args.max_lines {
        assert!(max > 0);
    }
    let mut match_cnt = 0;
    let re = args.build_regex();
    while let Some(line) = reader.read_line().await {
        match_cnt += get_matches(
            &re,
            line,
            &mut writer,
            args.first_match_only,
            args.first_capture_only,
            args.keep_unmatched,
        )
        .await;
        if let Some(max) = args.max_lines {
            if match_cnt >= max {
                debug!(
                    "stopping after {} lines (max {:?})",
                    match_cnt, args.max_lines
                );
                break;
            }
        }
    }
    Ok(match_cnt)
}

#[cfg(test)]
mod tests {
    use crate::common::CollectorWriter;
    use crate::common::VecReader;

    use super::*;

    async fn test_grab<S: Into<String>, T: Into<String>>(input: Vec<S>, expected: Vec<T>) {
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b".to_owned(),
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await
    }

    async fn test_grab_arg<S: Into<String>, T: Into<String>>(
        args: GrabArgs,
        input: Vec<S>,
        expected: Vec<T>,
    ) {
        let writer = CollectorWriter::new();
        let lines = writer.lines();
        grab(args, VecReader::new(input), writer).await.unwrap();
        let expected = expected
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<String>>();
        let line_vec = lines.snapshot().await;
        assert_eq!(&*line_vec, &expected)
    }

    #[async_std::test]
    async fn no_lines() {
        let empty: Vec<String> = vec![];
        test_grab(empty.clone(), empty).await;
    }

    #[async_std::test]
    async fn empty_line() {
        let expected: Vec<String> = vec![];
        test_grab(vec![""], expected).await;
    }

    #[async_std::test]
    async fn ignore_not_matching() {
        let expected: Vec<String> = vec![];
        test_grab(vec!["c"], expected).await;
    }

    #[async_std::test]
    async fn ignore_only_group_matches() {
        let expected: Vec<String> = vec![];
        test_grab(vec!["aa"], expected).await;
    }

    #[async_std::test]
    async fn match_single_group() {
        let expected: Vec<String> = vec!["aa".to_owned()];
        test_grab(vec!["aab"], expected).await;
    }

    #[async_std::test]
    async fn match_some_lines() {
        let expected: Vec<String> = vec!["aa".to_owned(), "a".to_owned(), "AA".to_owned()];
        test_grab(vec!["aab", "", "cab", "AAB"], expected).await;
    }

    #[async_std::test]
    async fn all_of_multi_per_line() {
        let expected: Vec<String> = vec!["aa".to_owned(), "a".to_owned()];
        test_grab(vec!["aabab"], expected).await;
    }

    #[async_std::test]
    async fn case_sensitive() {
        test_grab_arg(
            GrabArgs {
                pattern: "aa".to_owned(),
                case_sensitive: true,
                ..GrabArgs::default()
            },
            vec!["aa", "AA"],
            vec!["aa".to_owned()],
        ).await;
    }

    #[async_std::test]
    async fn case_insensitive() {
        test_grab_arg(
            GrabArgs {
                pattern: "aa".to_owned(),
                case_sensitive: false,
                ..GrabArgs::default()
            },
            vec!["aa", "AA"],
            vec!["aa".to_owned(), "AA".to_owned()],
        ).await;
    }

    #[async_std::test]
    async fn first_of_multi_per_line() {
        let expected: Vec<String> = vec!["aa".to_owned()];
        let input = vec!["aabab"];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b".to_owned(),
                first_match_only: true,
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn multiple_per_line() {
        let expected: Vec<String> = vec!["aa".to_owned(), "aaa".to_owned()];
        test_grab(vec!["aabbcaaabb"], expected).await;
    }

    #[async_std::test]
    async fn full_match_if_no_group() {
        let input = vec!["aab"];
        let expected: Vec<String> = vec!["aab".to_owned()];
        test_grab_arg(
            GrabArgs {
                pattern: "a+b".to_owned(),
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn match_multiple_groups() {
        let input = vec!["aabccd"];
        let expected: Vec<String> = vec!["aa".to_owned(), "cc".to_owned()];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b(c{2})".to_owned(),
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn multiple_matches_with_multiple_groups() {
        let input = vec!["aabccdabbca"];
        let expected: Vec<String> = vec!["aa".to_owned(), "cc".to_owned(), "a".to_owned()];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b+(c{2})?".to_owned(),
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn first_matches_with_first_groups() {
        // First match is just 'cb', the first group of which is empty.
        let input = vec!["cbdaavbb"];
        let expected: Vec<String> = vec![];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)?c(b+)?".to_owned(),
                first_match_only: true,
                first_capture_only: true,
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn match_only_first_groups() {
        let input = vec!["aabccd"];
        let expected: Vec<String> = vec!["aa".to_owned()];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b(c{2})".to_owned(),
                first_capture_only: true,
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn keep_unmatched_lines() {
        let input = vec!["aabccd", "abc", "bcc"];
        let expected: Vec<String> = vec![
            "aa".to_owned(),
            "cc".to_owned(),
            "abc".to_owned(),
            "bcc".to_owned(),
        ];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b(c{2})".to_owned(),
                keep_unmatched: true,
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    #[async_std::test]
    async fn first_group_and_keep_unmatched() {
        let input = vec!["aabccd", "abc", "bcc"];
        let expected: Vec<String> = vec!["aa".to_owned(), "abc".to_owned(), "bcc".to_owned()];
        test_grab_arg(
            GrabArgs {
                pattern: "(a+)b(c{2})".to_owned(),
                keep_unmatched: true,
                first_match_only: true,
                first_capture_only: true,
                max_lines: None,
                expect_match: false,
                expect_no_match: false,
                case_sensitive: false,
                ..GrabArgs::default()
            },
            input,
            expected,
        )
        .await;
    }

    //TODO @mverleg: test max lines
}
