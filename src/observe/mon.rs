use ::std::time::Instant;

use ::log::debug;

use crate::common::{LineWriter, PrefixWriter, StdWriter, Task, VecWriter};
use crate::ExitStatus;
use crate::observe::mon_args::MonArgs;
use crate::observe::sound_notification;

pub async fn mon(args: MonArgs, output_writer: &mut impl LineWriter, minitor_writer: &mut impl LineWriter) -> ExitStatus {
    let task = args.cmd.clone().into_task();
    if let Some(mut prefix) = args.prefix.clone() {
        assert!(!prefix.contains("%{date}"), "placeholders not supported yet for mon --prefix");
        assert!(!prefix.contains("%{time}"), "placeholders not supported yet for mon --prefix");
        prefix.push(' ');
        mon_task_with_writer(&task, args, &mut PrefixWriter::new(writer, prefix)).await
    } else {
        mon_task_with_writer(&task, args, writer).await
    }

}

pub async fn mon_task_with_writer(task: &Task, args: MonArgs, writer: &mut impl LineWriter) -> ExitStatus {
    mon_task(
        &task,
        output_writer,
        minitor_writer,
        !args.no_print_cmd,
        !args.no_output_on_success,
        !args.no_timing,
        args.sound_success,
        args.sound_failure,
    ).await
}

pub async fn mon_task(
    task: &Task,
    output_writer: &mut impl LineWriter,
    monitor_writer: &mut impl LineWriter,
    print_cmd: bool,
    output_on_success: bool,
    timing: bool,
    sound_success: bool,
    sound_failure: bool,
) -> ExitStatus {
    debug!("print_cmd={print_cmd} output_on_success={output_on_success} timing={timing} sound_success={sound_success} sound_failure={sound_failure} for task {}", task.as_str());
    let cmd_str = task.as_str();
    if print_cmd {
        monitor_writer.write_line(format!("going to run {}", cmd_str)).await;
    }
    let t0 = Instant::now();
    let status = if output_on_success {
        let mut err_writer = StdWriter::stderr();
        task.execute_with_stdout_nomonitor(output_writer, &mut err_writer)
            .await
    } else {
        debug!("mon buffering output, will show on error");
        let mut out_buffer = VecWriter::new();
        let mut err_writer = StdWriter::stderr();
        let status = task
            .execute_with_stdout_nomonitor(&mut out_buffer, &mut err_writer)
            .await;
        if status.is_err() {
            eprintln!("printing all output because process failed");
            output_writer.write_all_lines(out_buffer.get().iter()).await;
        }
        status
    };
    let duration = t0.elapsed().as_millis();
    if timing {
        if status.is_ok() {
            if cmd_str.len() > 256 {
                // approximate for non-ascii
                monitor_writer
                    .write_line(format!(
                        "success: took {} ms to run {}...",
                        duration,
                        cmd_str.chars().take(256).collect::<String>()
                    ))
                    .await;
            } else {
                monitor_writer
                    .write_line(format!("success: took {} ms to run {}", duration, cmd_str))
                    .await;
            }
        } else {
            eprintln!(
                "FAILED command {} in {} ms (code {})",
                cmd_str,
                duration,
                status.code()
            );
        }
    }
    if let Err(err) = sound_notification(sound_success, sound_failure, status.is_ok()) {
        eprintln!("notification sound problem: {}", err);
        return ExitStatus::err();
    }
    status
}
