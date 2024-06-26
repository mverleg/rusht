use ::std::fmt::Debug;
use ::std::io::stdin;
use ::std::io::BufReader;
use ::std::io::Stdin;
use ::std::io::BufRead;
use ::std::path::Path;

use ::async_std::fs::File;
use ::async_std::io::prelude::BufReadExt;
use ::async_trait::async_trait;
use ::log::debug;

use crate::common::async_gate::AsyncGate;

#[async_trait]
pub trait LineReader: Debug + Send {
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

impl Default for StdinReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineReader for StdinReader {
    async fn read_line(&mut self) -> Option<&str> {
        self.buffer.clear();
        let read_len = self.reader.read_line(&mut self.buffer).unwrap();
        while self.buffer.ends_with('\n') || self.buffer.ends_with('\r') {
            self.buffer.pop();
        }
        if read_len == 0 {
            return None;
        }
        Some(&self.buffer)
    }
}

#[derive(Debug)]
pub struct FileReader {
    reader: async_std::io::BufReader<File>,
    buffer: String,
}

impl FileReader {
    pub async fn new(path: &Path) -> Self {
        let file = match File::open(path).await {
            Ok(file) => file,
            Err(err) => panic!("could not open file '{}', err {:?}", path.to_string_lossy(), err),
        };
        FileReader {
            reader: async_std::io::BufReader::new(file),
            buffer: String::with_capacity(256),
        }
    }
}

#[async_trait]
impl LineReader for FileReader {
    async fn read_line(&mut self) -> Option<&str> {
        self.buffer.clear();
        let read_len = self.reader.read_line(&mut self.buffer).await.unwrap();
        while self.buffer.ends_with('\n') || self.buffer.ends_with('\r') {
            self.buffer.pop();
        }
        if read_len == 0 {
            return None;
        }
        Some(&self.buffer)
    }
}

#[derive(Debug)]
pub struct RejectStdin {
    gate: AsyncGate,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum StdinWaitResult {
    Data,
    Completed,
}

impl RejectStdin {
    pub fn new() -> Self {
        debug!("disabled stdin rejection in {}:{}", file!(), line!());
        // let gate = AsyncGate::new();
        // let gate_clone = gate.clone();
        // let f = async move || {
        //     debug!("starting monitor to reject stdin input");
        //     let res = async_std::io::stdin()
        //         .read(&mut [0])
        //         .map(|_| StdinWaitResult::Data)
        //         .race(gate_clone.wait().map(|_| StdinWaitResult::Completed))
        //         .await;
        //     if res == StdinWaitResult::Data {
        //         eprintln!("received data on stdin but did not expect any");
        //         exit(1);
        //     }
        //     debug!("finished stdin rejection monitor because the reader was dropped");
        // };
        // async_std::task::spawn(f());
        // RejectStdin { gate }
        unimplemented!()
    }
}

impl Drop for RejectStdin {
    fn drop(&mut self) {
        self.gate.open(true);
    }
}

#[async_trait]
impl LineReader for RejectStdin {
    async fn read_line(&mut self) -> Option<&str> {
        todo!() //TODO @mverleg: TEMPORARY! REMOVE THIS!
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

#[derive(Debug)]
pub struct NonEmptyLineReader<'a, R: LineReader> {
    delegate: &'a mut R,
    current: String,
}

impl <'a, R: LineReader> NonEmptyLineReader<'a, R> {
    pub fn wrap(delegate_reader: &'a mut R) -> Self {
        NonEmptyLineReader {
            delegate: delegate_reader,
            current: "".to_owned(),
        }
    }
}

#[async_trait]
impl <'a, R: LineReader> LineReader for NonEmptyLineReader<'a, R> {
    async fn read_line(&mut self) -> Option<&str> {
        while let Some(line) = self.delegate.read_line().await {
            if ! line.trim().is_empty() {
                self.current = line.to_owned();
                return Some(&self.current)
            }
        }
        None
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
