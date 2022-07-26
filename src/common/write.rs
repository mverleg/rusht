use ::async_std::io::stdout;
use ::async_std::io::Stdout;
use ::async_std::io::WriteExt;
use ::async_std::sync::Arc;
use ::async_std::sync::Mutex;
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

impl Default for StdoutWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineWriter for StdoutWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let bytes = line.as_ref().as_bytes();
        let expected = bytes.len();
        let write_len = self.writer.write(bytes).await.unwrap();
        assert_eq!(expected, write_len);
        assert_eq!(1, self.writer.write(&[b'\n']).await.unwrap()); //TODO @mverleg: more efficient way with single await?
    }
}

#[derive(Debug)]
pub struct VecWriter {
    lines: Vec<String>,
}

impl VecWriter {
    pub fn new() -> Self {
        VecWriter { lines: vec![] }
    }

    pub fn get(self) -> Vec<String> {
        self.lines
    }
}

impl Default for VecWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineWriter for VecWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.lines.push(line.as_ref().to_owned())
    }
}

#[derive(Debug)]
pub struct CollectorWriter {
    lines: Arc<Mutex<Vec<String>>>,
}

impl CollectorWriter {
    pub fn new() -> Self {
        CollectorWriter {
            lines: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_lines(&self) -> Arc<Mutex<Vec<String>>> {
        self.lines.clone()
    }
}

impl Default for CollectorWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineWriter for CollectorWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.lines.lock().await.push(line.as_ref().to_owned())
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

impl Default for FirstItemWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineWriter for FirstItemWriter {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        self.line.get_or_insert_with(|| line.as_ref().to_owned());
    }
}
