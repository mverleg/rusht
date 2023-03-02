use log::debug;
use crate::common::LineReader;
use crate::common::LineWriter;
use crate::filter::between_args::MatchHandling;
use crate::filter::BetweenArgs;

pub async fn between(args: BetweenArgs, reader: &mut impl LineReader, writer: &mut impl LineWriter) {
    // Search start point
    let mut i = 0;
    while let Some(line) = reader.read_line().await {
        if args.from.is_match(line) {
            debug!("found a 'between' start match at line #{i}, handling={}", args.from_handling);
            if args.from_handling == MatchHandling::Include {
                writer.write_line(line);
            }
            break
        }
        i += 1;
    }
    // Search end point
    while let Some(line) = reader.read_line().await {
        i += 1;
    }
    // Skip the rest
}
