use ::std::collections::HashMap;
use ::std::env;
use ::std::io::{BufRead, BufReader};
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::ExitStatus as ProcStatus;
use ::std::process::Stdio;
use ::std::time::Instant;

use ::async_std::task::block_on;
use ::clap::StructOpt;
use ::dashmap::DashMap;
use ::itertools::Itertools;
use ::lazy_static::lazy_static;
use ::log::{debug, warn};
use ::log::info;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::which::which_all;

use crate::common::{fail, LineWriter, StdWriter};

lazy_static! {
    static ref EXE_CACHE: DashMap<String, String> = DashMap::new();
}

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
        let full_cmd = resolve_executable(&cmd);
        Task::new_with_env(full_cmd, args, working_dir, HashMap::new())
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
        Task::new(cmd.to_owned(), args.to_vec(), env::current_dir().unwrap())
    }

    pub fn new_split(parts: Vec<String>, working_dir: PathBuf) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        Task::new(cmd.to_owned(), args.to_vec(), working_dir)
    }

    pub fn push_arg(&mut self, extra_arg: &str) {
        self.args.push(extra_arg.to_owned());
    }

    pub fn as_cmd_str(&self) -> String {
        if self.args.is_empty() {
            format!("{}", self.cmd)
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
        format!("{}{}{}", env_str, self.as_cmd_str(), cmd_str,)
    }

    pub fn execute(&self, quiet: bool) -> ProcStatus {
        block_on(self.execute_with_stdout(quiet, &mut StdWriter::stdout()))
        //TODO @mverleg: block on for now since it does not have suspend points anyway
    }

    pub async fn execute_with_stdout(
        &self,
        quiet: bool,
        writer: &mut impl LineWriter,
    ) -> ProcStatus {
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
        let mut line = String::new();
        loop {
            line.clear();
            match out.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    line.pop(); // strip newline  //TODO @mverleg: cross-platform?
                    writer.write_line(&line).await
                }
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
                if cmd_str.len() > 256 {  // approximate for non-ascii
                    println!("took {} ms to run {}...", duration,
                             cmd_str.chars().take(256).collect::<String>());
                } else {
                    println!("took {} ms to run {}", duration, cmd_str);
                }
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

fn resolve_executable(base_cmd: &str) -> String {
    if let Some(cached_exe) = EXE_CACHE.get(base_cmd) {
        debug!("using cached executable {} for {}", cached_exe.value(), base_cmd);
        return cached_exe.value().clone()
    }
    let do_warn = !env::var("RUSHT_SUPPRESS_EXE_RESOLVE").is_ok();
    let mut full_cmds = which_all(base_cmd)
        .unwrap_or_else(|err| panic!("error while trying to find command '{}' on path, err: {}", base_cmd, err));
    let full_cmd: String = match full_cmds.next() {
        Some(cmd) => {
            if let Some(more_cmd) = full_cmds.next() {
                if do_warn {
                    info!("more than one command found for {}: {} and {} (choosing the first)", base_cmd, cmd.to_string_lossy(), more_cmd.to_string_lossy())
                }
            }
            cmd.to_str().unwrap_or_else(|| panic!("command {} executable {} not unicode", base_cmd, cmd.to_string_lossy())).to_owned()
        },
        None => {
            if do_warn {
                warn!("could not find executable for {}, will try to run anyway", base_cmd);
            }
            base_cmd.to_owned()
        }
    };
    debug!("using executable {} for {}", &full_cmd, base_cmd);
    full_cmd
}
