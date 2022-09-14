extern crate core;

use ::std::cmp::max;

use ::async_std::channel::Sender;
use ::async_std::channel::{bounded, Receiver};
use ::async_std::task::block_on;
use ::log::debug;

use ::async_trait::async_trait;

use crate::common::{LineReader, LineWriter};

//TODO @mverleg: maybe could be more efficient using single-line buffer and custom wakers, like AsyncGate

#[derive(Debug, PartialEq, Eq)]
enum PipeItem {
    Line(String),
    End,
}

#[derive(Debug)]
pub struct ChainWriter {
    sender: Sender<PipeItem>,
}

#[async_trait]
impl LineWriter for ChainWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let line = line.as_ref().to_owned();
        debug!("chain write: {}", &line); //TODO @mark: TEMPORARY! REMOVE THIS!
        self.sender.send(PipeItem::Line(line)).await.unwrap()
    }
}

impl Drop for ChainWriter {
    fn drop(&mut self) {
        // TODO rewrite for async drop if supported
        debug!("ending chain writer pipe"); //TODO @mark: TEMPORARY! REMOVE THIS!
        block_on(self.sender.send(PipeItem::End)).unwrap()
    }
}

#[derive(Debug)]
pub struct ChainReader {
    receiver: Receiver<PipeItem>,
    current: PipeItem,
}

#[async_trait]
impl LineReader for ChainReader {
    async fn read_line(&mut self) -> Option<&str> {
        debug!("chain read start"); //TODO @mark: TEMPORARY! REMOVE THIS!
        if PipeItem::End == self.current {
            debug!("chain reader pipe was already closed"); //TODO @mark: TEMPORARY! REMOVE THIS!
            return None;
        }
        self.current = self.receiver.recv().await.unwrap();
        match &self.current {
            PipeItem::Line(line) => Some(line),
            PipeItem::End => {
                debug!("chain reader pipe was just closed"); //TODO @mark: TEMPORARY! REMOVE THIS!
                None
            }
        }
    }
}

pub fn chained(buffer_size: usize) -> (ChainWriter, ChainReader) {
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

#[cfg(test)]
mod tests {
    use ::std::future::join;

    use ::regex::Regex;

    use crate::common::{CollectorWriter, VecReader};
    use crate::filter::{grab, unique, GrabArgs, Keep, Order, UniqueArgs};
    use crate::observe::chain::chained;

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
            first_match_only: true,
            first_capture_only: true,
            keep_unmatched: true,
            max_lines: None,
        };

        let mut out2 = CollectorWriter::new();
        let lines = out2.lines();
        let unique_args = UniqueArgs {
            order: Order::Preserve,
            keep: Keep::First,
            by: Some(Regex::new("([^ ])* ").unwrap()),
            prefix: false,
        };
        let (res, ()) = join!(
            //TODO @mark: probably an easier way for this:
            grab(grab_args, inp1, out1),
            unique(unique_args, &mut inp2, &mut out2),
        )
        .await;
        res.unwrap();

        let expected = vec!["world", "Mars", "Venus", "bye world"];
        let actual = lines.snapshot().await;
        assert_eq!(&*actual, &expected);
    }
}
