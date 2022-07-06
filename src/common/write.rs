use ::async_std::io::Stdout;
use ::async_std::io::stdout;
use ::async_std::io::WriteExt;
use ::async_trait::async_trait;

#[async_trait]
pub trait LineWriter: Send {
    async fn write_line(&mut self, line: impl AsRef<str> + Send);

    async fn write_all_lines<S: AsRef<str> + Send>(&mut self, lines: impl Iterator<Item=S> + Send) {
        for line in lines {
            self.write_line(line).await
        }
    }
}

#[derive(Debug)]
pub struct StdoutWriter {
    writer: Stdout,
}

impl StdoutWriter {
    pub fn new() -> Self {
        StdoutWriter {
            writer: stdout(),
        }
    }
}

#[async_trait]
impl LineWriter for StdoutWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let expected = line.as_ref().as_bytes().len();
        let bytes = line.as_ref().as_bytes();
        let write_len = self.writer.write(bytes).await.unwrap();
        assert_eq!(expected, write_len);
    }
}

#[derive(Debug)]
pub struct VecWriter {
    lines: Vec<String>,
}

impl VecWriter {
    pub fn new() -> Self {
        VecWriter {
            lines: vec![]
        }
    }

    pub fn get(self) -> Vec<String> {
        self.lines
    }

    pub fn assert_eq<S: Into<String>>(&self, lines: Vec<S>) {
        let expected: Vec<String> = lines.into_iter()
            .map(|line| line.into())
            .collect();
        assert_eq!(self.lines, expected);
    }
}

#[async_trait]
impl LineWriter for VecWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.lines.push(line.as_ref().to_owned())
    }
}

#[derive(Debug)]
pub struct FirstItemWriter {
    line: Option<String>,
}

impl FirstItemWriter {
    pub fn new() -> Self {
        FirstItemWriter {
            line: None
        }
    }

    pub fn get(self) -> Option<String> {
        self.line
    }
}

#[async_trait]
impl LineWriter for FirstItemWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.line.get_or_insert_with(|| line.as_ref().to_owned());
    }
}
