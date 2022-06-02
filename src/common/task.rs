use ::std::env::current_dir;
use ::std::io::{BufRead, BufReader, Read};
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::ExitStatus;
use ::std::process::Stdio;
use ::std::thread;
use ::std::time::Instant;

use ::serde::Deserialize;
use ::serde::Serialize;
use ::structopt::StructOpt;

use crate::common::fail;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum CommandArgs {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

impl CommandArgs {
    pub fn unpack(self) -> Vec<String> {
        match self {
            CommandArgs::Cmd(cmd) => cmd,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

impl Task {
    pub fn new(cmd: String, args: Vec<String>, working_dir: PathBuf) -> Self {
        Task {
            cmd,
            args,
            working_dir,
        }
    }

    pub fn new_split_in_cwd(parts: Vec<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), current_dir().unwrap())
    }

    pub fn new_split(parts: Vec<String>, working_dir: PathBuf) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), working_dir)
    }

    pub fn as_cmd_str(&self) -> String {
        format!("{} {}", self.cmd, self.args.join(" "))
    }

    pub fn execute(&self, quiet: bool) -> ExitStatus {
        self.execute_with_out_err(quiet, |line| {
            println!("{}", line)
        }, |line| {
            eprintln!("{}", line)
        })
    }

    pub fn execute_with_out_err(
        &self, quiet: bool,
        out_line_handler: impl FnMut(&str) + Send + 'static,
        err_line_handler: impl FnMut(&str) + Send + 'static,
    ) -> ExitStatus {
        let t0 = Instant::now();
        let cmd_str = self.as_cmd_str();
        let mut child = match Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => fail(format!(
                "failed to start command '{}', error {}",
                cmd_str, err
            )),
        };
        thread::spawn(move || continuous_reader(child.stdout.unwrap(), out_line_handler));
        thread::spawn(move || continuous_reader(child.stderr.unwrap(), err_line_handler));
        let status = match child.wait() {
            Ok(status) => status,
            Err(err) => fail(format!(
                "failed to finish command '{}', error {}",
                cmd_str, err
            )),
        };
        if !quiet {
            let duration = t0.elapsed().as_millis();
            println!("took {} ms to run: {}", duration, cmd_str);
        }
        status
    }
}

fn continuous_reader(readable: impl Read, mut handler: impl FnMut(&str)) {
    let mut out = BufReader::new(readable);
    let mut line = String::new();
    loop {
        line.clear();
        match out.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => handler(&line),
            Err(err) => panic!("failed to read output line, err: {}", err),
        }
    }
}