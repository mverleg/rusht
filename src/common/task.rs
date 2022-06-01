use ::std::process::Command;
use ::std::process::Stdio;
use ::std::time::Instant;
use std::process::ExitStatus;

use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::fail;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
}

impl Task {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Task { cmd, args }
    }

    pub fn new_split(parts: Vec<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec())
    }

    pub fn as_cmd_str(&self) -> String {
        format!("{} {}", self.cmd, self.args.join(" "))
    }

    pub fn execute(&self, quiet: bool) -> ExitStatus {
        let t0 = Instant::now();
        let cmd_str = self.as_cmd_str();
        let mut child = match Command::new(&self.cmd)
            .args(&self.args)
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
