use ::std::fmt::Write;
use ::std::future::join;
use std::io::Stderr;

use ::async_std::io;
use ::async_std::io::Stdout;
use ::async_std::io::WriteExt;
use ::async_std::sync::Arc;
use ::async_std::sync::Mutex;
use ::async_std::sync::MutexGuard;
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

//TODO @mverleg: not called Std?
#[derive(Debug)]
pub struct StdWriter<W: Write + Send> {
    writer: W,
}

impl <W: Write + Send> StdWriter<W> {
    pub fn of(writer: W) -> Self {
        StdWriter { writer }
    }

    pub fn stdout() -> StdWriter<io::Stdout> {
        StdWriter::of(io::stdout())
    }

    pub fn stderr() -> StdWriter<io::Stderr> {
        StdWriter::of(io::stderr())
    }
}

impl <W: Write + Send> Default for StdWriter<W> {
    fn default() -> Self {
        Self::stdout()
    }
}

#[async_trait]
impl <W: Write + Send> LineWriter for StdWriter<W> {
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
    lines: LineContainer,
}

#[derive(Debug, Clone)]
pub struct LineContainer {
    lines: Arc<Mutex<Vec<String>>>,
}

impl LineContainer {
    pub async fn snapshot(&self) -> MutexGuard<Vec<String>> {
        self.lines.lock().await
    }
}

impl CollectorWriter {
    pub fn new() -> Self {
        CollectorWriter {
            lines: LineContainer {
                lines: Arc::new(Mutex::new(vec![])),
            },
        }
    }

    pub fn lines(&self) -> LineContainer {
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
        self.lines.lines.lock().await.push(line.as_ref().to_owned())
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

#[derive(Debug)]
pub struct TeeWriter<'a, W1: LineWriter, W2: LineWriter> {
    first: &'a mut W1,
    second: &'a mut W2,
}

impl<'a, W1: LineWriter, W2: LineWriter> TeeWriter<'a, W1, W2> {
    pub fn new(first: &'a mut W1, second: &'a mut W2) -> Self {
        TeeWriter { first, second }
    }
}

#[async_trait]
impl<'a, W1: LineWriter, W2: LineWriter> LineWriter for TeeWriter<'a, W1, W2> {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let line = line.as_ref();
        let _: ((), ()) = join!(self.first.write_line(line), self.second.write_line(line),).await;
    }
}
