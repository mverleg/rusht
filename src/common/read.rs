use ::async_std::io::BufReader;
use ::async_std::io::prelude::BufReadExt;
use ::async_std::io::stdin;
use ::async_std::io::Stdin;
use ::async_trait::async_trait;

#[async_trait]
trait LineReader {
    async fn read_line(&mut self) -> Option<&str>;
}

#[derive(Debug)]
struct StdinReader {
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
struct VecReader {
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
