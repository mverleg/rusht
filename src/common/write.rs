use std::cell::RefCell;
use ::async_std::io::stdout;
use ::async_std::io::Stdout;
use ::async_std::io::WriteExt;
use ::async_trait::async_trait;

#[async_trait]
pub trait LineWriter: Send {
    async fn write_line(&mut self, line: impl AsRef<str> + Send);

    async fn write_all_lines<S: AsRef<str> + Send>(
        &mut self,
        lines: impl Iterator<Item = S> + Send,
    ) {
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
        StdoutWriter { writer: stdout() }
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

#[derive(Debug, Clone)]
pub struct VecWriter {
    lines: RefCell<Vec<String>>,
}

impl VecWriter {
    pub fn new() -> Self {
        VecWriter { lines: RefCell::new(vec![]) }
    }

    pub fn get(&self) -> &[String] {
        self.lines.borrow()
    }

    pub fn get_mut(&mut self) -> &mut [String] {
        self.lines.borrow_mut()
    }

    pub fn assert_eq<S: Into<String>>(&self, lines: Vec<S>) {
        let expected: Vec<String> = lines.into_iter().map(|line| line.into()).collect();
        assert_eq!(&*self.lines.borrow(), &expected);
    }
}

#[async_trait]
impl LineWriter for VecWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.lines.borrow_mut().push(line.as_ref().to_owned())
    }
}

#[derive(Debug)]
pub struct FirstItemWriter {
    line: Option<String>,
}

impl FirstItemWriter {
    pub fn new() -> Self {
        FirstItemWriter { line: None }
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
