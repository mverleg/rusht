use ::std::collections::HashMap;
use ::std::env;
use ::std::io::{BufRead, BufReader};
use ::std::iter;
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::Stdio;
use ::std::thread;
use ::std::time::Instant;
use std::io::Read;
use std::process::ChildStdout;

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
use crate::ExitStatus;
use crate::observe::mon_task;

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
        format!("{}{}{}", env_str, self.as_cmd_str(), cmd_str,)
    }

    pub fn execute_sync(&self, monitor: bool) -> ExitStatus {
        let writer = &mut StdWriter::stdout();
        block_on(self.execute_with_stdout(monitor, writer))
    }

    pub async fn execute_with_stdout(
        &self,
        monitor: bool,
        writer: &mut impl LineWriter,
    ) -> ExitStatus {
        if monitor {
            mon_task(self, writer, true, true, true, false, true).await
        } else {
            self.execute_with_stdout_nomonitor(writer).await
        }
    }

    pub async fn execute_with_stdout_nomonitor(&self, writer: &mut impl LineWriter) -> ExitStatus {
        let use_shell_env = "RUSHT_SHELL_EXEC";
        if env::var(use_shell_env).is_ok() {
            debug!("using shell execution mode (because {use_shell_env} is set); this is inexplicably much faster for mvn, but may cause escaping issues");
            let mut cmd = Command::new("sh");
            let joined_cmd = iter::once(format!("'{}'", self.cmd))
                .chain(self.args.iter()
                    .inspect(|arg| if arg.contains("'") {
                        panic!("argument {} should not contain single quote in shell mode ({})", arg, use_shell_env)
                    })
                    .map(|arg| format!("'{}'", arg))
                ).join(" ");
            cmd.args(&["-c".to_owned(), joined_cmd]);
            self.execute_cmd_with_stdout(cmd, writer).await
        } else {
            debug!("not using shell execution mode (because {use_shell_env} is not set); this is the safe way but may be slower");
            let mut cmd = Command::new(&self.cmd);
            cmd.args(&self.args);
            self.execute_cmd_with_stdout(cmd, writer).await
        }
    }

    async fn execute_cmd_with_stdout(&self, mut base_cmd: Command, out_writer: &mut impl LineWriter) -> ExitStatus {
        // Note: it is complex to read both stdout and stderr (https://stackoverflow.com/a/34616729)
        // even with threading so for now do only the stdout.
        debug!("command to run: '{}' {}", base_cmd.get_program().to_string_lossy(),
            base_cmd.get_args().map(|a| format!("\"{}\"", a.to_string_lossy())).join(" "));
        let mut child = match base_cmd
            .current_dir(&self.working_dir)
            .envs(&self.extra_envs)
            //.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => fail(format!(
                "failed to start command '{}', error {}",
                self.as_cmd_str(),
                err
            )),
        };
        //let mut inp = BufReader::new(child.stdin.take().unwrap());
        let q: impl async_std::io::Read = child.stdout.take().unwrap();
        let mut out: BufReader<ChildStdout> = BufReader::new(child.stdout.take().unwrap());
        let cmd_str = self.as_str();
        let cmd2writer = CommandToWriter::new();
        thread::scope(move |_scope| cmd2writer.run());
        let status = match child.wait() {
            Ok(status) => status,
            Err(err) => fail(format!(
                "failed to finish command '{}', error {}",
                self.as_cmd_str(),
                err
            )),
        };
        ExitStatus::of_code(status.code())
    }
}

struct CommandToWriter<R: Read> {
    proc_out: BufReader<R>,
}

impl <R: Read> CommandToWriter<R> {
    pub fn new(reader: R) -> Self {
        CommandToWriter {
            proc_out: BufReader::new(reader)
        }
    }

    pub fn run(&mut self) {
        //TODO @mverleg: ideally this should use some natively async Process, instead of threads
        let mut line = String::new();
        loop {
            line.clear();
            match self.proc_out.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    while line.ends_with('\n') || line.ends_with('\r') {
                        line.pop();
                    }
                    block_on(out_writer.write_line(&line))
                }
                Err(err) => panic!(
                    "failed to read output of the task, task: {}, err: {}",
                    cmd_str,
                    err
                ),
            }
        }
    }
}

fn resolve_executable(base_cmd: String) -> String {
    let t0 = Instant::now();
    if base_cmd.contains('/') {
        debug!(
            "command {} appears to already be a path, not resolving further",
            base_cmd
        );
        return base_cmd;
    }
    if let Some(cached_exe) = EXE_CACHE.get(&base_cmd) {
        debug!(
            "using cached executable {} for {}",
            cached_exe.value(),
            base_cmd
        );
        return cached_exe.value().clone();
    }
    let do_warn = env::var("RUSHT_SUPPRESS_EXE_RESOLVE").is_err();
    let full_cmd: String = {
        let mut full_cmds = which_all(&base_cmd).unwrap_or_else(|err| {
            panic!(
                "error while trying to find command '{}' on path, err: {}",
                base_cmd, err
            )
        });
        match full_cmds.next() {
            Some(cmd) => {
                if let Some(more_cmd) = full_cmds.next() {
                    if do_warn {
                        info!(
                            "more than one command found for {}: {} and {} (choosing the first)",
                            base_cmd,
                            cmd.to_string_lossy(),
                            more_cmd.to_string_lossy()
                        )
                    }
                }
                cmd.to_str()
                    .unwrap_or_else(|| {
                        panic!(
                            "command {} executable {} not unicode",
                            base_cmd,
                            cmd.to_string_lossy()
                        )
                    })
                    .to_owned()
            }
            None => {
                if do_warn {
                    warn!(
                        "could not find executable for {}, will try to run anyway",
                        base_cmd
                    );
                }
                base_cmd.clone()
            }
        }
    };
    debug!("caching executable {} for {}", &full_cmd, &base_cmd);
    EXE_CACHE.insert(base_cmd, full_cmd.clone());
    let duration = t0.elapsed().as_millis();
    if duration > 200 {
        warn!("resolve_executable slow, took {} ms", duration);
    } else {
        debug!("resolve_executable took {} ms", duration);
    }
    full_cmd
}
