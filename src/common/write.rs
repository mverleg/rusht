use ::std::future::join;
use ::std::io;
use ::std::io::Write;
use ::std::sync::atomic::{AtomicU64, Ordering}; // using async caused deadlocks in concurrent mvn commands

use ::async_std::sync::Arc;
use ::async_std::sync::Mutex;
use ::async_std::sync::MutexGuard;
use ::async_trait::async_trait;
use ::log::debug;
use ::regex::Regex;
use ::smallvec::SmallVec;

static DEBUG_NR: AtomicU64 = AtomicU64::new(0); //TODO @mverleg:

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
pub struct StdWriter<W: Write + Unpin + Send> {
    writer: W,
}

impl<W: Write + Unpin + Send> StdWriter<W> {
    pub fn of(writer: W) -> Self {
        StdWriter { writer }
    }
}

impl StdWriter<io::Stdout> {
    pub fn stdout() -> Self {
        StdWriter::of(io::stdout())
    }
}

impl StdWriter<io::Stderr> {
    pub fn stderr() -> Self {
        StdWriter::of(io::stderr())
    }
}

#[async_trait]
impl<W: Write + Unpin + Send> LineWriter for StdWriter<W> {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let bytes = line.as_ref().as_bytes();
        let expected = bytes.len();
        let nr = DEBUG_NR.fetch_add(1, Ordering::AcqRel); //TODO @mverleg: TEMPORARY! REMOVE THIS!
        debug!(
            "{} before writing {} bytes to async std (out?): {}",
            nr,
            bytes.len(),
            line.as_ref()
        ); //TODO @mverleg: TEMPORARY! REMOVE THIS!
        let write_len = self.writer.write(bytes).unwrap();
        debug!(
            "{} after writing {} bytes to async std (out?): {}",
            nr,
            bytes.len(),
            line.as_ref()
        ); //TODO @mverleg: TEMPORARY! REMOVE THIS!
        assert_eq!(expected, write_len);
        assert_eq!(1, self.writer.write(&[b'\n']).unwrap()); //TODO @mverleg: more efficient way with single await?
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
pub struct DiscardWriter();

impl DiscardWriter {
    pub fn new() -> Self {
        DiscardWriter()
    }
}

impl Default for DiscardWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineWriter for DiscardWriter {
    async fn write_line(&mut self, _line: impl AsRef<str> + Send) {
        // drop line
    }
}

/// For every line written, send it to two other writers.
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

/// Several handles can send to the same writer asynchronously.
#[derive(Debug)]
pub struct FunnelWriter<'a, W: LineWriter> {
    name: &'a str,
    delegate: Arc<Mutex<&'a mut W>>,
}

#[derive(Debug)]
pub struct FunnelFactory<'a, W: LineWriter> {
    delegate: Arc<Mutex<&'a mut W>>,
}

impl<'a, W: LineWriter> FunnelFactory<'a, W> {
    pub fn new(delegate: &'a mut W) -> Self {
        FunnelFactory {
            delegate: Arc::new(Mutex::new(delegate)),
        }
    }

    pub fn writer(&self, name: &'a str) -> FunnelWriter<'a, W> {
        FunnelWriter {
            name,
            delegate: self.delegate.clone(),
        }
    }
}

#[async_trait]
impl<'a, W: LineWriter> LineWriter for FunnelWriter<'a, W> {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let line = line.as_ref();
        let mut dw = self.delegate.lock().await;
        dw.write_line(format!("[{}] {}", self.name, line)).await;
    }
}

/// For every line written, send it to two other writers.
#[derive(Debug)]
pub struct RegexWatcherWriter<F: Fn(&str) + Send> {
    patterns: SmallVec<[Regex; 1]>,
    action: F,
}

impl<F: Fn(&str) + Send> RegexWatcherWriter<F> {
    pub fn new(patterns: impl Into<SmallVec<[Regex; 1]>>, action: F) -> Self {
        RegexWatcherWriter {
            patterns: patterns.into(),
            action,
        }
    }
}

#[async_trait]
impl<F: Fn(&str) + Send> LineWriter for RegexWatcherWriter<F> {
    async fn write_line(&mut self, line: impl AsRef<str> + Send) {
        let line = line.as_ref();
        for pattern in &self.patterns {
            if pattern.is_match(line) {
                debug!("pattern {} matched for line {}", pattern, line);
                (self.action)(line)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_craete_writer_without_type_annotation() {
        let _ = StdWriter::stdout();
    }
}
