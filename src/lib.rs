#![feature(future_join)]
#![feature(future_poll_fn)]
#![feature(async_closure)]

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
    use ::std::future::join;

    use ::async_std::channel::{Receiver, Sender};
    use ::async_std::channel::bounded;
    use ::async_std::task::block_on;
    use ::async_trait::async_trait;
    use ::regex::Regex;

    use crate::common::{CollectorWriter, LineReader, LineWriter, VecReader};
    use crate::filter::{grab, GrabArgs, Keep, Order, unique, UniqueArgs};

    #[async_std::test]
    async fn chain_inout() {
        let inp1 = VecReader::new(vec![
            "hello world",
            "hello Mars",
            "hello Venus",
            "bye world",
            "bye Jupiter",
        ]);
        let (out1, mut inp2) = chained(1);

        let grab_args = GrabArgs {
            pattern: Regex::new("^hello (.*)").unwrap(),
            first_only: true,
            keep_unmatched: true,
        };

        let mut out2 = CollectorWriter::new();
        let lines = out2.get_lines();
        let unique_args = UniqueArgs {
            order: Order::Preserve,
            keep: Keep::First,
            by: None,
            prefix: true,
        };
        join!(
            //TODO @mark: probably an easier way for this:
            (async || grab(grab_args, inp1, out1).await.unwrap())(),
            unique(unique_args, &mut inp2, &mut out2),
        ).await;

        let expected = vec!["world", "Mars", "Venus", "bye"];
        let actual = &*lines.lock().await;
        assert_eq!(actual, &expected);
    }

    #[derive(Debug, PartialEq, Eq)]
    enum PipeItem {
        Line(String),
        End,
    }

    #[derive(Debug)]
    struct ChainWriter {
        sender: Sender<PipeItem>,
    }

    #[async_trait]
    impl LineWriter for ChainWriter {
        async fn write_line(&mut self, line: impl AsRef<str> + Send) {
            let line = line.as_ref().to_owned();
            eprintln!("chain write: {}", &line);  //TODO @mark: TEMPORARY! REMOVE THIS!
            self.sender.send(PipeItem::Line(line)).await.unwrap()
        }
    }

    impl Drop for ChainWriter {
        fn drop(&mut self) {
            // TODO rewrite for async drop if supported
            block_on(self.sender.send(PipeItem::End)).unwrap()
        }
    }

    #[derive(Debug)]
    struct ChainReader {
        receiver: Receiver<PipeItem>,
        current: PipeItem,
    }

    #[async_trait]
    impl LineReader for ChainReader {
        async fn read_line(&mut self) -> Option<&str> {
            eprintln!("chain read start");  //TODO @mark: TEMPORARY! REMOVE THIS!
            if PipeItem::End == self.current {
                return None
            }
            self.current = self.receiver.recv().await.unwrap();
            match &self.current {
                PipeItem::Line(line) => Some(line),
                PipeItem::End => None,
            }
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
                current: PipeItem::Line("".to_string()),
            },
        )
    }
}
