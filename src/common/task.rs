use ::std::env::current_dir;
use ::std::io::{BufRead, BufReader};
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::ExitStatus;
use ::std::process::Stdio;
use ::std::time::Instant;

use ::serde::Deserialize;
use ::serde::Serialize;
use ::clap::StructOpt;

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

    pub fn into_task(self) -> Task {
        Task::new_split_in_cwd(self.unpack())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn as_str(&self) -> String {
        if self.working_dir == current_dir().unwrap() {
            self.as_cmd_str()
        } else {
            format!("{} @ {}", self.as_cmd_str(), self.working_dir.to_string_lossy())
        }
    }

    pub fn execute(&self, quiet: bool) -> ExitStatus {
        self.execute_with_stdout(quiet, |line| println!("{}", line))
    }

    pub fn execute_with_stdout(
        &self,
        quiet: bool,
        mut out_line_handler: impl FnMut(&str),
    ) -> ExitStatus {
        // Note: it is complex to read both stdout and stderr (https://stackoverflow.com/a/34616729)
        // even with threading so for now do only the stdout.
        let t0 = Instant::now();
        let cmd_str = self.as_str();
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
        let mut out = BufReader::new(child.stdout.take().unwrap());
        loop {
            let mut line = String::new();
            match out.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => out_line_handler(&line),
                Err(err) => panic!(
                    "failed to read output of the task, task: {}, err: {}",
                    self.as_str(),
                    err
                ),
            }
        }
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
