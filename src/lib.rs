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

    use super::*;

    #[test]
    fn implement_test() {
        let inp = VecReader::new(vec![
            "hello world",
            "hello Mars",
            "hello Venus",
            "bye world",
        ]);
        let out = VecWriter::new();
        grab(GrabArgs {
            pattern: Regex::new("^hello ").unwrap(),
            first_only: true,
            keep_unmatched: true
        }, inp, out);

    }
}
