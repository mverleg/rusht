use ::std::collections::HashMap;
use ::std::env;
use ::std::iter;
use ::std::path::PathBuf;
use ::std::thread;

use ::async_std::io::BufReader;
use ::async_std::io::prelude::BufReadExt;
use ::async_std::process::Command;
use ::async_std::process::Stdio;
use ::async_std::task::block_on;
use ::itertools::Itertools;
use ::log::debug;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::async_std::io::Read;

use crate::common::{LineWriter, resolve_executable, StdWriter};
use crate::ExitStatus;
use crate::observe::mon_task;

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
        let full_cmd = resolve_executable(cmd);
        Task {
            cmd: full_cmd,
            args,
            working_dir,
            extra_envs,
        }
    }

    pub fn new_split_in_cwd(parts: Vec<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), env::current_dir().unwrap())
    }

    pub fn new_split(parts: Vec<String>, working_dir: PathBuf) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), working_dir)
    }

    #[cfg(test)]
    pub fn noop() -> Self {
        Task::new("true".to_owned(), vec![], env::current_dir().unwrap())
    }

    pub fn push_arg(&mut self, extra_arg: &str) {
        self.args.push(extra_arg.to_owned());
    }

    pub fn as_cmd_str(&self) -> String {
        if self.args.is_empty() {
            self.cmd.to_string()
        } else {
            format!("{} {}", self.cmd, self.args.join(" "))
        }
    }

    pub fn as_str(&self) -> String {
        let cmd_str = if self.working_dir == env::current_dir().unwrap() {
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
        format!("{}{}{}", env_str, self.as_cmd_str(), cmd_str, )
    }
}
