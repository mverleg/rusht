use ::async_std::io::BufReader;
use ::async_std::io::stdin;
use ::async_std::io::Stdin;
use ::async_std::io::prelude::BufReadExt;

trait LineReader {
    fn read_line(&mut self) -> Option<&str>;
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

impl LineReader for StdinReader {
    fn read_line(&mut self) -> Option<&str> {
        self.reader.read_line(&mut self.buffer);
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

impl LineReader for VecReader {
    fn read_line(&mut self) -> Option<&str> {
        self.current = self.lines.pop()?;
        Some(&self.current)
    }
}
