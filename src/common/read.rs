use ::async_std::io::BufReader;
use ::async_std::io::prelude::BufReadExt;
use ::async_std::io::stdin;
use ::async_std::io::Stdin;
use ::async_trait::async_trait;

use crate::common::write::LineWriter;

#[async_trait]
pub trait LineReader {
    async fn read_line(&mut self) -> Option<&str>;
}

#[derive(Debug)]
struct LineReaderIterator<'a, R: LineReader> {
    reader: &'a R
}

impl <'a, R: LineReader> Iterator for LineReaderIterator<'a, R> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.reader.read_line().await
    }
}

// impl <'a> IntoIterator for LineReader {
//     type Item = &'a str;
//     type IntoIter = ReaderIterator;
//
//     fn into_iter(self) -> ReaderIterator {
//         ReaderIterator
//     }
// }

#[derive(Debug)]
pub struct StdinReader {
    reader: BufReader<Stdin>,
    buffer: String,
}

impl StdinReader {
    pub fn new() -> Self {
        StdinReader {
            reader: BufReader::new(stdin()),
            buffer: String::with_capacity(256),
        }
    }
}

#[async_trait]
impl LineReader for StdinReader {
    async fn read_line(&mut self) -> Option<&str> {
        let read_len = self.reader.read_line(&mut self.buffer).await.unwrap();
        assert_eq!(read_len, self.buffer.len());
        Some(&self.buffer)
    }
}

#[derive(Debug)]
pub struct VecReader {
    lines: Vec<String>,
    current: String,
}

impl VecReader {
    pub fn new<S: Into<String>>(mut lines: Vec<S>) -> Self {
        lines.reverse();
        VecReader {
            lines: lines.into_iter()
                .map(|line| line.into())
                .collect(),
            current: "".to_string(),
        }
    }
}

#[async_trait]
impl LineReader for VecReader {
    async fn read_line(&mut self) -> Option<&str> {
        self.current = self.lines.pop()?;
        Some(&self.current)
    }
}

// #[derive(Debug)]
// pub struct ChainReader<W: LineWriter> {
//     writer: W,
// }
//
// impl <W: LineWriter> ChainReader<W> {
//     pub fn new<S: Into<String>>(writer: W) -> Self {
//         ChainReader {
//             writer
//         }
//     }
// }
//
// #[async_trait]
// impl <W: LineWriter> LineReader for ChainReader<W> {
//     async fn read_line(&mut self) -> Option<&str> {
//         self.writer.write_line()
//     }
// }
