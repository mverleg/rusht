use ::std::process::Command;
use ::std::process::Stdio;
use ::std::time::Instant;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::ExitStatus;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

impl Task {
    pub fn new(cmd: String, args: Vec<String>, working_dir: PathBuf) -> Self {
        Task { cmd, args, working_dir }
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
        let t0 = Instant::now();
        let cmd_str = self.as_cmd_str();
        dbg!(&self.working_dir);  //TODO @mark: TEMPORARY! REMOVE THIS!
        let mut child = match Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => fail(format!(
                "failed to start command '{}', error {}",
                cmd_str, err
            )),
        };
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
