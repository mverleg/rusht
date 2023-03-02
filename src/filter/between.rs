use log::debug;

use crate::common::LineReader;
use crate::common::LineWriter;
use crate::filter::between_args::MatchHandling;
use crate::filter::BetweenArgs;

pub async fn between(args: BetweenArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) {
    // Search start point
    let mut i = 1;
    let mut found_start = false;
    while let Some(line) = reader.read_line().await {
        if args.from.is_match(line) {
            debug!("found a 'between' start match at line #{i}, handling={}", args.from_handling);
            found_start = true;
            if args.from_handling == MatchHandling::Include {
                writer.write_line(line).await;
            }
            break;
        }
        i += 1;
    }
    if ! found_start {
        debug!("reached end of input in 'between' after {i} lines before finding start match; stopping");
        return
    }
    if let Some(end_re) = &args.to {
        debug!("searching end pattern in 'between' from line #{i}");
        while let Some(line) = reader.read_line().await {
            if end_re.is_match(line) {
                debug!("found a 'between' end match at line #{i}, handling={}", args.to_handling);
                if args.to_handling == MatchHandling::Include {
                    writer.write_line(line).await;
                }
                return;
            }
            writer.write_line(line).await;
            i += 1;
        }
        debug!("reached end of input in 'between' before finding end match after line #{i}");
    } else {
        debug!("no end pattern in 'between', returning all remaining lines from line #{i}");
        while let Some(line) = reader.read_line().await {
            writer.write_line(line).await;
            i += 1;
        }
    }
    // Skip the rest
}

#[cfg(test)]
mod tests {
    use ::regex::Regex;

    use crate::common::{CollectorWriter, VecReader};

    use super::*;

    async fn check_between<L: Into<String>>(lines: Vec<L>) -> Vec<String> {
        let mut writer = CollectorWriter::new();
        let args = BetweenArgs {
            from: Regex::new("start").unwrap(),
            to: Some(Regex::new("end").unwrap()),
            from_handling: MatchHandling::Include,
            to_handling: MatchHandling::Exclude,
        };
        between(args, &mut VecReader::new(lines), &mut writer).await;
        writer.lines().snapshot().await.clone()
    }

    #[async_std::test]
    async fn start_match() {
        env_logger::init();  //TODO @mverleg: TEMPORARY! REMOVE THIS!
        let res = check_between(vec!["before", "start", "middle"]).await;
        assert_eq!(res, vec!["start", "middle"]);
    }

    #[async_std::test]
    async fn end_match() {
        let res = check_between(vec!["middle", "end", "after"]).await;
        assert!(res.is_empty());
    }

    #[async_std::test]
    async fn no_start_or_end_match() {
        let res = check_between(vec!["before", "middle", "after"]).await;
        assert!(res.is_empty());
    }

    #[async_std::test]
    async fn start_and_end_match() {
        let res = check_between(vec!["before", "start", "middle", "end", "after"]).await;
        assert_eq!(res, vec!["start", "middle"]);
    }

    #[async_std::test]
    async fn start_and_end_same_line_match() {
        // For now the behaviour is that this does not detect end, mostly because it is easier
        let res = check_between(vec!["before", "start end", "after"]).await;
        assert_eq!(res, vec!["start end", "after"]);
    }
}