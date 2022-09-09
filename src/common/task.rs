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
        format!("{}{}{}", env_str, self.as_cmd_str(), cmd_str,)
    }

    pub fn execute_sync(&self, monitor: bool) -> ExitStatus {
        let writer = &mut StdWriter::stdout();
        block_on(self.execute_with_stdout(monitor, writer))
    }

    pub async fn execute_with_stdout(
        &self,
        monitor: bool,
        out_writer: &mut impl LineWriter,
    ) -> ExitStatus {
        let mut err_writer = StdWriter::stderr();
        self.execute_with_outerr(monitor, out_writer, &mut err_writer).await
    }

    pub async fn execute_with_outerr(
        &self,
        monitor: bool,
        out_writer: &mut impl LineWriter,
        err_writer: &mut impl LineWriter,
    ) -> ExitStatus {
        if monitor {
            mon_task(self, out_writer, true, true, true, false, true).await
        } else {
            self.execute_with_stdout_nomonitor(out_writer, err_writer).await
        }
    }

    pub async fn execute_with_stdout_nomonitor(&self,
        out_writer: &mut impl LineWriter,
        err_writer: &mut impl LineWriter,
    ) -> ExitStatus {
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
            self.execute_cmd_with_outerr(cmd, out_writer, err_writer).await.unwrap()
            //TODO @mverleg: get rid of unwrap
        } else {
            debug!("not using shell execution mode (because {use_shell_env} is not set); this is the safe way but may be slower");
            let mut cmd = Command::new(&self.cmd);
            cmd.args(&self.args);
            self.execute_cmd_with_outerr(cmd, out_writer, err_writer).await.unwrap()
            //TODO @mverleg: get rid of unwrap
        }
    }

    async fn execute_cmd_with_outerr(&self,
        mut base_cmd: Command,
        out_writer: &mut impl LineWriter,
        err_writer: &mut impl LineWriter,
    ) -> Result<ExitStatus, String> {
        // note: cannot log with async_std because it does not expose getters on Command
        // debug!("command to run: '{}' {}", base_cmd.get_program().to_string_lossy(),
        //     base_cmd.get_args().map(|a| format!("\"{}\"", a.to_string_lossy())).join(" "));
        let mut child = base_cmd
            .current_dir(&self.working_dir)
            .envs(&self.extra_envs)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("failed to start command '{}', error {}", self.as_cmd_str(), err))?;

        // This uses threads because async_std spawn did not have scoped tasks, so writer needs to be 'static, which it is not
        thread::scope(move |scope| {
            let proc_out = child.stdout.take().unwrap();
            let proc_err = child.stderr.take().unwrap();
            let out_task = scope.spawn(move || block_on(forward_out(proc_out, out_writer)));
            let err_task = scope.spawn(move || block_on(forward_out(proc_err, err_writer)));
            //TODO @mverleg: only do status() after stdin is closed, otherwise it closes it
            let status = block_on(child.status())
                .map_err(|err| format!("failed to finish command '{}', error {}", self.as_cmd_str(), err))?;
            out_task.join().expect("thread panic")?;
            err_task.join().expect("thread panic")?;
            Ok(ExitStatus::of_code(status.code()))
        })
    }
}

async fn forward_out(stdout: impl Read + Unpin, writer: &mut impl LineWriter) -> Result<(), String> {
    let mut out_buf = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match out_buf.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }
                writer.write_line(&line).await
            }
            Err(err) => return Err(format!(
                "failed to read, err: {}",
                err
            )),
        }
    }
    Ok(())
}
