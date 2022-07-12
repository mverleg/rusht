pub mod cached;
pub mod cmd;
pub mod common;
pub mod filter;
pub mod find;
pub mod wait;
pub mod escape;

#[cfg(test)]
mod tests {
    use ::std::cmp::max;

    use ::async_std::channel::{Receiver, Sender};
    use ::async_std::channel::bounded;
    use ::async_trait::async_trait;
    use ::futures::future::join;
    use ::futures::FutureExt;
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
        let (mut out1, mut inp2) = chained(1);

        let grab_args = GrabArgs {
            pattern: Regex::new("^hello (.*)").unwrap(),
            first_only: true,
            keep_unmatched: true,
        };

        let mut out2 = VecWriter::new();
        let unique_args = UniqueArgs {
            order: Order::Preserve,
            keep: Keep::First,
            by: None,
            prefix: true,
        };
        join(
            grab(grab_args, &mut inp1, &mut out1).map(|v| v.unwrap()),
            unique(unique_args, &mut inp2, &mut out2),
        ).await;

        out2.assert_eq(vec!["world", "Mars", "Venus", "bye"]);
    }

    #[derive(Debug)]
    struct ChainWriter {
        sender: Sender<String>,
    }

    #[async_trait]
    impl LineWriter for ChainWriter {
        async fn write_line(&mut self, line: impl AsRef<str> + Send) {
            let line = line.as_ref().to_owned();
            eprintln!("chain write: {}", &line);  //TODO @mark: TEMPORARY! REMOVE THIS!
            self.sender.send(line).await.unwrap()
        }
    }

    #[derive(Debug)]
    struct ChainReader {
        receiver: Receiver<String>,
        current: String,
    }

    #[async_trait]
    impl LineReader for ChainReader {
        async fn read_line(&mut self) -> Option<&str> {
            eprintln!("chain read start");  //TODO @mark: TEMPORARY! REMOVE THIS!
            self.current = self.receiver.recv().await.unwrap();
            eprintln!("chain read done: {}", &self.current);  //TODO @mark: TEMPORARY! REMOVE THIS!
            Some(&self.current)
        }
    }

    //TODO @mark: move to common read/write
    fn chained(buffer_size: usize) -> (ChainWriter, ChainReader) {
        let buffer_size = max(2, buffer_size);
        let (sender, receiver) = bounded(buffer_size);
        (
            ChainWriter { sender },
            ChainReader {
                receiver,
                current: "".to_string(),
            },
        )
    }
}
