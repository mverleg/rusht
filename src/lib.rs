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

    #[test]
    fn implement_test() {
        let mut inp = VecReader::new(vec![
            "hello world",
            "hello Mars",
            "hello Venus",
            "bye world",
        ]);
        let mut out = VecWriter::new();
        grab(GrabArgs {
            pattern: Regex::new("^hello ").unwrap(),
            first_only: true,
            keep_unmatched: true
        },
             &mut inp,
             &mut out);

    }
}
