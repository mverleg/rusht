//TODO @mverleg: possibly to be replaced by `exec2`

use ::std::env;
use ::std::iter;
use ::std::thread;

use ::async_std::io as aio;
use ::async_std::process::Command;
use ::async_std::process::Stdio;
use ::async_std::task::block_on;
use ::futures::AsyncBufReadExt;
use ::itertools::Itertools;
use ::log::debug;
use futures::AsyncWriteExt;

use crate::common::write::FunnelFactory;
use crate::common::{LineWriter, StdWriter, Task};
use crate::observe::mon_task;
use crate::ExitStatus;

static USE_SHELL_ENV_NAME: &'static str = "RUSHT_SHELL_EXEC";

impl Task {
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
        self.execute_with_outerr(monitor, out_writer, &mut err_writer)
            .await
    }

    pub async fn execute_with_outerr(
        &self,
        monitor: bool,
        out_writer: &mut impl LineWriter,
        err_writer: &mut impl LineWriter,
    ) -> ExitStatus {
        if monitor {
            let funnel = FunnelFactory::new(out_writer);
            mon_task(
                self,
                &mut funnel.writer(""),
                &mut funnel.writer(""),
                true,
                true,
                true,
                true,
                false,
                false,
            )
            .await
        } else {
            self.execute_with_stdout_nomonitor(out_writer, err_writer)
                .await
        }
    }

    pub async fn execute_with_stdout_nomonitor(
        &self,
        out_writer: &mut impl LineWriter,
        err_writer: &mut impl LineWriter,
    ) -> ExitStatus {
        if env::var(USE_SHELL_ENV_NAME).is_ok() {
            debug!("using shell execution mode (because {USE_SHELL_ENV_NAME} is set); this is inexplicably much faster for mvn, but may cause escaping issues");
            let mut cmd = Command::new("sh");
            let joined_cmd = iter::once(format!("'{}'", self.cmd))
                .chain(self.args.iter()
                    .inspect(|arg| if arg.contains('\'') {
                        panic!("argument {} should not contain single quote in shell mode ({USE_SHELL_ENV_NAME})", arg)
                    })
                    .map(|arg| format!("'{}'", arg))
                ).join(" ");
            cmd.args(&["-c".to_owned(), joined_cmd]);
            self.execute_cmd_with_outerr(cmd, out_writer, err_writer)
                .await
                .unwrap()
            //TODO @mverleg: get rid of unwrap
        } else {
            debug!("not using shell execution mode (because {USE_SHELL_ENV_NAME} is not set); this is the safe way but may be slower");
            let mut cmd = Command::new(&self.cmd);
            cmd.args(&self.args);
            self.execute_cmd_with_outerr(cmd, out_writer, err_writer)
                .await
                .unwrap()
            //TODO @mverleg: get rid of unwrap
        }
    }

    async fn execute_cmd_with_outerr(
        &self,
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
            .stdin(if self.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                format!(
                    "failed to start command '{}', error {}",
                    self.as_cmd_str(),
                    err
                )
            })?;

        // This uses threads because async_std spawn did not have scoped tasks, so writer needs to be 'static, which it is not
        thread::scope(move |scope| {
            let proc_out = child.stdout.take().unwrap();
            let proc_err = child.stderr.take().unwrap();
            let out_task = scope.spawn(move || forward_out(proc_out, out_writer));
            let err_task = scope.spawn(move || forward_out(proc_err, err_writer));
            let in_task = if let Some(sin) = &self.stdin {
                let mut proc_in = child.stdin.take().expect("child should have stdin piped");
                Some(scope.spawn(move || {
                    block_on(proc_in.write_all(sin.as_bytes())).expect("failed to send stdin")
                }))
            } else {
                None
            };
            //TODO @mverleg: only do status() after stdin is closed, otherwise it closes it
            let status = block_on(child.status()).map_err(|err| {
                format!(
                    "failed to finish command '{}', error {}",
                    self.as_cmd_str(),
                    err
                )
            })?;
            out_task.join().expect("thread panic")?;
            err_task.join().expect("thread panic")?;
            in_task.map(|it| it.join().expect("thread panic"));
            Ok(ExitStatus::of_code(status.code()))
        })
    }
}

fn forward_out(stdout: impl aio::Read + Unpin, writer: &mut impl LineWriter) -> Result<(), String> {
    let mut out_buf = aio::BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match block_on(out_buf.read_line(&mut line)) {
            Ok(0) => break,
            Ok(_) => {
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }
                block_on(writer.write_line(&line))
            }
            Err(err) => return Err(format!("failed to read, err: {}", err)),
        }
    }
    Ok(())
}
