use ::std::collections::HashMap;
use ::std::env::current_dir;
use ::std::future::Future;
use ::std::io::{BufRead, BufReader};
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::ExitStatus as ProcStatus;
use ::std::process::Stdio;
use ::std::time::Instant;

use ::clap::StructOpt;
use ::itertools::Itertools;
use ::serde::Deserialize;
use ::serde::Serialize;
use async_std::task::block_on;

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
    #[serde(default)]
    pub extra_envs: HashMap<String, String>,
}

impl Task {
    pub fn new(cmd: String, args: Vec<String>, working_dir: PathBuf) -> Self {
        Task::new_with_env(cmd, args, working_dir, HashMap::new())
    }

    pub fn new_with_env(
        cmd: String,
        args: Vec<String>,
        working_dir: PathBuf,
        extra_envs: HashMap<String, String>,
    ) -> Self {
        Task {
            cmd,
            args,
            working_dir,
            extra_envs,
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

    pub fn push_arg(&mut self, extra_arg: &str) {
        self.args.push(extra_arg.to_owned());
    }

    pub fn as_cmd_str(&self) -> String {
        format!("{} {}", self.cmd, self.args.join(" "))
    }

    pub fn as_str(&self) -> String {
        let cmd_str = if self.working_dir == current_dir().unwrap() {
            "".to_owned()
        } else {
            format!(" @ {}", self.working_dir.to_string_lossy())
        };
        let env_str = if self.extra_envs.is_empty() {
            "".to_owned()
        } else {
            format!(
                "{} ",
                self.extra_envs
                    .iter()
                    .map(|(k, v)| format!("{}='{}'", k, v))
                    .join(" ")
            )
        };
        format!("{}{}{}", env_str, self.as_cmd_str(), cmd_str,)
    }

    pub fn execute(&self, quiet: bool) -> ProcStatus {
        block_on(self.execute_with_stdout(quiet, &mut async move |line| print!("{}", line)))
        //TODO @mverleg: block on for now since it does not have suspend points anyway
    }

    pub async fn execute_with_stdout<Fut>(
        &self,
        quiet: bool,
        async_out_line_handler: impl FnMut(&str) -> Fut,
    ) -> ProcStatus
    where Fut: Future<Output = ()> {
        // Note: it is complex to read both stdout and stderr (https://stackoverflow.com/a/34616729)
        // even with threading so for now do only the stdout.
        let t0 = Instant::now();
        let cmd_str = self.as_str();
        let mut child = match Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .envs(&self.extra_envs)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
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
                Ok(_) => async_out_line_handler(&line).await,
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
            if status.success() {
                println!("command {} successfully ran in {} ms", cmd_str, duration);
            } else {
                eprintln!(
                    "command {} FAILED in {} ms (code {})",
                    cmd_str,
                    duration,
                    status.code().unwrap_or(-1)
                );
            }
        }
        status
    }
}
