use ::std::collections::HashMap;
use ::std::env;
use ::std::fmt::Write;
use ::std::path::PathBuf;

use ::itertools::Itertools;
use ::lazy_static::lazy_static;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;
use dirs::home_dir;

use crate::common::resolve_executable;

lazy_static! {
    static ref SAFE_ARG_RE: Regex = Regex::new(r"^[\p{L}0-9_\-\.,@/:=\+]+$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    #[serde(default)]
    pub stdin: Option<String>,
    #[serde(default)]
    pub extra_envs: HashMap<String, String>,
}

impl Task {
    pub fn new(
        cmd: String,
        args: Vec<String>,
        working_dir: PathBuf,
        stdin: Option<String>,
    ) -> Self {
        Task::new_with_env(cmd, args, working_dir, stdin, HashMap::new())
    }

    pub fn new_in_cwd(cmd: String, stdin: Option<String>, args: Vec<String>) -> Self {
        Task::new(cmd, args, env::current_dir().unwrap(), stdin)
    }

    pub fn new_with_env(
        cmd: String,
        args: Vec<String>,
        working_dir: PathBuf,
        stdin: Option<String>,
        extra_envs: HashMap<String, String>,
    ) -> Self {
        let full_cmd = resolve_executable(cmd);
        Task {
            cmd: full_cmd,
            args,
            working_dir,
            stdin,
            extra_envs,
        }
    }

    pub fn new_split_in_cwd(parts: Vec<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(
            cmd.to_owned(),
            args.to_vec(),
            env::current_dir().unwrap(),
            None,
        )
    }

    pub fn new_split(parts: Vec<String>, working_dir: PathBuf, stdin: Option<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), working_dir, stdin)
    }

    #[cfg(test)]
    pub fn noop() -> Self {
        Task::new("true".to_owned(), vec![], env::current_dir().unwrap(), None)
    }

    pub fn push_arg(&mut self, extra_arg: &str) {
        self.args.push(extra_arg.to_owned());
    }

    pub fn as_cmd_str(&self) -> String {
        let mut txt = String::new();
        if let Some(home) = home_dir() {
            let cmd = PathBuf::from(&self.cmd);
            if let Ok(rel_pth) = cmd.strip_prefix(&home) {
                txt.push_str("~/");
                txt.push_str(rel_pth.to_str().expect("path should be utf8"))
            }
        }
        if txt.is_empty() {
            txt.push_str(&self.cmd)
        }
        for arg in &self.args {
            if SAFE_ARG_RE.is_match(arg) {
                write!(txt, " {}", arg).unwrap()
            } else {
                write!(txt, " '{}'", arg).unwrap()
            }
        }
        txt
    }

    pub fn as_str(&self) -> String {
        let mut cmd_str = if let Some(sin) = &self.stdin {
            format!(" <{}b", sin.len())
        } else {
            "".to_owned()
        };
        if self.working_dir != env::current_dir().unwrap() {
            write!(cmd_str, " @ {}", self.working_dir.to_string_lossy()).unwrap()
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
}
