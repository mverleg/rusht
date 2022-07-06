pub mod cached;
pub mod cmd;
pub mod common;
pub mod filter;
pub mod find;
pub mod wait;

#[cfg(test)]
mod tests {
    use ::std::cmp::max;

    use ::async_spsc::spsc;
    use ::async_trait::async_trait;
    use ::regex::Regex;

    use crate::common::{LineReader, LineWriter, VecReader, VecWriter};
    use crate::filter::{grab, GrabArgs, Keep, Order, unique, UniqueArgs};

    #[async_std::test]
    async fn chain_inout() {
        let mut inp1 = VecReader::new(vec![
            "hello world",
            "hello Mars",
            "hello Venus",
            "bye world",
            "bye Jupiter",
        ]);
        let (mut out1, mut inp2) = chained();

        let grab_args = GrabArgs {
            pattern: Regex::new("^hello (.*)").unwrap(),
            first_only: true,
            keep_unmatched: true
        };
        grab(grab_args, &mut inp1, &mut out1).await.unwrap();

        let mut out2 = VecWriter::new();
        let unique_args = UniqueArgs {
            order: Order::Preserve,
            keep: Keep::First,
            by: None,
            prefix: true
        };
        unique(unique_args, &mut inp2, &mut out2).await;

        out2.assert_eq(vec![
            "world",
            "Mars",
            "Venus",
            "bye",
        ]);
    }

    #[derive(Debug)]
    struct ChainWriter {}

    #[async_trait]
    impl LineWriter for ChainWriter {
        async fn write_line(&mut self, line: impl AsRef<str> + Send) {
            todo!()
        }
    }

    #[derive(Debug)]
    struct ChainReader {}

    #[async_trait]
    impl LineReader for ChainReader {
        async fn read_line(&mut self) -> Option<&str> {
            todo!()
        }
    }

    //TODO @mark: move to common read/write
    fn chained(mut buffer_size: u32) -> (ChainWriter, ChainReader) {
        buffer_size = max(1, buffer_size);
        let (sender, receiver) = spsc::<String>(buffer_size);
        (ChainWriter { sender }, ChainReader { receiver })
    }
}
