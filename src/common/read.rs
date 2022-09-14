use std::thread;
use ::async_std::io::prelude::BufReadExt;
use ::async_std::io::stdin;
use ::async_std::io::BufReader;
use ::async_std::io::Stdin;
use ::async_trait::async_trait;
use async_std::prelude::FutureExt as AltExt;
use futures::{AsyncReadExt, FutureExt};
use crate::common::async_gate::AsyncGate;

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

impl Default for StdinReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LineReader for StdinReader {
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

#[derive(Debug, PartialEq)]
enum StdinWaitResult { DATA, COMPLETED }

impl RejectStdin {
    pub fn new() -> Self {
        let gate = AsyncGate::new();
        let gate_clone = gate.clone();
        thread::spawn(async move || {
            let res = async_std::io::stdin().read(&mut [0]).map(|_| StdinWaitResult::DATA).race(
                gate_clone.wait().map(|_| StdinWaitResult::COMPLETED)).await;
            if res == StdinWaitResult::DATA {
                eprintln!("received data on stdin but did not expect any");
                panic!("unexpected stdin");
            }
        });
        RejectStdin { gate }
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
