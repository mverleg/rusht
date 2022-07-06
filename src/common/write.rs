use ::async_std::io::Stdout;
use ::async_std::io::stdout;
use async_std::io::WriteExt;

trait LineWriter {
    fn write_line(&mut self, line: impl AsRef<str>);
}

#[derive(Debug)]
struct StdoutWriter {
    writer: Stdout,
}

impl StdoutWriter {
    pub fn new() -> Self {
        StdoutWriter {
            writer: stdout(),
        }
    }
}

impl LineWriter for StdoutWriter {
    fn write_line(&mut self, line: impl AsRef<str>) {
        self.writer.write(line.as_ref().as_bytes());
    }
}

#[derive(Debug)]
struct VecWriter {
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
}

impl LineWriter for VecWriter {
    fn write_line(&mut self, line: impl AsRef<str>) {
        self.lines.push(line.as_ref().to_owned())
    }
}
