use ::std::env;
use ::std::io;
use ::std::iter;
use ::std::thread;

use ::async_std::io as aio;
use ::async_std::process::Command;
use ::async_std::process::Stdio;
use ::async_std::task::block_on;
use ::futures::AsyncBufReadExt;
use ::itertools::Itertools;
use ::log::debug;

use crate::common::{LineReader, LineWriter, RejectStdin, StdWriter, Task};
use crate::observe::mon_task;
use crate::ExitStatus;

#[derive(Debug)]
pub struct ExecutionBuilder<'a, I, O, E>
where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    task: &'a Task,
    inp: Option<&'a mut I>,
    out: Option<&'a mut O>,
    err: Option<&'a mut E>,
    monitor: bool,
}

impl<'a> ExecutionBuilder<'a, RejectStdin, StdWriter<io::Stdout>, StdWriter<io::Stderr>> {
    pub fn of(task: &'a Task) -> Self {
        ExecutionBuilder {
            task,
            inp: None,
            out: None,
            err: None,
            monitor: false,
        }
    }
}

impl<'a, I, O, E> ExecutionBuilder<'a, I, O, E>
where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    pub fn inp<I2: LineReader>(self, inp: &'a mut I2) -> ExecutionBuilder<'a, I2, O, E> {
        ExecutionBuilder {
            inp: Some(inp),
            ..self
        }
    }

    pub fn out<O2: LineWriter>(self, out: &'a mut O2) -> ExecutionBuilder<'a, I, O2, E> {
        ExecutionBuilder {
            out: Some(out),
            ..self
        }
    }

    pub fn err<E2: LineWriter>(self, err: &'a mut E2) -> ExecutionBuilder<'a, I, O, E2> {
        ExecutionBuilder {
            err: Some(err),
            ..self
        }
    }

    pub fn start(self) {
        exec_open_inp(self.task, self.inp, self.out, self.err, self.monitor)
    }
}

fn exec_open_inp<I, O, E>(
    task: &Task,
    inp: Option<&mut I>,
    out: Option<&mut O>,
    err: Option<&mut E>,
    monitor: bool,
) where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    if let Some(inp) = inp {
        exec_open_out(task, inp, out, err, monitor)
    } else {
        exec_open_out(task, &mut RejectStdin::new(), out, err, monitor)
    }
}

fn exec_open_out<I, O, E>(
    task: &Task,
    inp: &mut I,
    out: Option<&mut O>,
    err: Option<&mut E>,
    monitor: bool,
) where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    if let Some(out) = out {
        exec_open_err(task, inp, out, err, monitor)
    } else {
        exec_open_err(task, inp, &mut StdWriter::stdout(), err, monitor)
    }
}

fn exec_open_err<I, O, E>(task: &Task, inp: &mut I, out: &mut O, err: Option<&mut E>, monitor: bool)
where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    if let Some(err) = err {
        exec_ioe(task, inp, out, err, monitor)
    } else {
        exec_ioe(task, inp, out, &mut StdWriter::stderr(), monitor)
    }
}

fn exec_ioe<I, O, E>(_task: &Task, _inp: &mut I, _out: &mut O, _err: &mut E, _monitor: bool)
where
    I: LineReader,
    O: LineWriter,
    E: LineWriter,
{
    todo!() //TODO @mverleg: TEMPORARY! REMOVE THIS!
}

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
            mon_task(self, out_writer, true, true, true, false, true).await
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
        let use_shell_env = "RUSHT_SHELL_EXEC";
        if env::var(use_shell_env).is_ok() {
            debug!("using shell execution mode (because {use_shell_env} is set); this is inexplicably much faster for mvn, but may cause escaping issues");
            let mut cmd = Command::new("sh");
            let joined_cmd = iter::once(format!("'{}'", self.cmd))
                .chain(self.args.iter()
                    .inspect(|arg| if arg.contains('\'') {
                        panic!("argument {} should not contain single quote in shell mode ({})", arg, use_shell_env)
                    })
                    .map(|arg| format!("'{}'", arg))
                ).join(" ");
            cmd.args(&["-c".to_owned(), joined_cmd]);
            self.execute_cmd_with_outerr(cmd, out_writer, err_writer)
                .await
                .unwrap()
            //TODO @mverleg: get rid of unwrap
        } else {
            debug!("not using shell execution mode (because {use_shell_env} is not set); this is the safe way but may be slower");
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

#[cfg(test)]
mod tests {
    use crate::common::read::RejectStdin;
    use crate::common::VecWriter;

    use super::*;

    #[test]
    #[ignore] //TODO @mark: TEMPORARY! REMOVE THIS!
    fn build_exec() {
        let task = Task::noop();
        ExecutionBuilder::of(&task)
            .inp(&mut RejectStdin::new())
            .out(&mut VecWriter::new())
            .err(&mut VecWriter::new())
            .start();
    }
}
