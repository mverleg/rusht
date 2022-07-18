use ::async_std::io::prelude::BufReadExt;
use ::async_std::io::stdin;
use ::async_std::io::BufReader;
use ::async_std::io::Stdin;
use ::async_trait::async_trait;

#[async_trait]
pub trait LineReader: Send {
    async fn read_line(&mut self) -> Option<&str>;

    async fn collect_all(&mut self) -> Vec<String> {
        let mut all = vec![];
        while let Some(line) = self.read_line().await {
            all.push(line.to_owned())
        }
        all
    }
}

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
        self.buffer.clear();
        let read_len = self.reader.read_line(&mut self.buffer).await.unwrap();
        if read_len == 0 {
            return None
        }
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
            lines: lines.into_iter().map(|line| line.into()).collect(),
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
