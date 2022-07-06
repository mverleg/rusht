pub mod cached;
pub mod cmd;
pub mod common;
pub mod filter;
pub mod find;
pub mod wait;

#[cfg(test)]
mod tests {
    use regex::Regex;

    use crate::common::{VecReader, VecWriter};
    use crate::filter::{grab, GrabArgs};

    #[async_std::test]
    async fn chain_inout() {
        let mut inp = VecReader::new(vec![
            "hello world",
            "hello Mars",
            "hello Venus",
            "bye world",
            "bye Jupiter",
        ]);
        let mut out = VecWriter::new();
        let grab_args = GrabArgs {
            pattern: Regex::new("^hello (.*)").unwrap(),
            first_only: true,
            keep_unmatched: true
        };
        grab(grab_args, &mut inp, &mut out).await.unwrap();
        out.assert_eq(vec![
            "world",
            "Mars",
            "Venus",
            "bye world",
        ]);
    }
}
