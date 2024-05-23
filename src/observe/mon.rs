use ::std::time::Instant;

use ::log::debug;
use time::OffsetDateTime;

use crate::common::Task;
use crate::common::StdWriter;
use crate::common::PrefixWriter;
use crate::common::LineWriter;
use crate::common::VecWriter;
use crate::observe::mon_args::MonArgs;
use crate::observe::sound_notification;
use crate::ExitStatus;

pub async fn mon(
    args: MonArgs,
    output_writer: &mut impl LineWriter,
    monitor_writer: &mut impl LineWriter,
) -> ExitStatus {
    let task = args.cmd.clone().into_task();
    if let Some(mut prefix) = args.prefix.clone() {
        assert!(
            !prefix.contains("%{date}"),
            "placeholders not supported yet for mon --prefix"
        );
        assert!(
            !prefix.contains("%{time}"),
            "placeholders not supported yet for mon --prefix"
        );
        prefix.push(' ');
        mon_task_with_writer(
            &task,
            args,
            &mut PrefixWriter::new(output_writer, prefix),
            monitor_writer,
        ).await
    } else {
        mon_task_with_writer(&task, args, output_writer, monitor_writer).await
    }
}

pub async fn mon_task_with_writer(
    task: &Task,
    args: MonArgs,
    output_writer: &mut impl LineWriter,
    monitor_writer: &mut impl LineWriter,
) -> ExitStatus {
    mon_task(
        &task,
        output_writer,
        monitor_writer,
        !args.no_print_cmd,
        args.full_command,
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
    full_cmd: bool,
    output_on_success: bool,
    timing: bool,
    sound_success: bool,
    sound_failure: bool,
) -> ExitStatus {
    let cmd_str = if full_cmd {
        task.as_str()
    } else {
        task.as_short_cmd_str()
    };
    if print_cmd {
        monitor_writer
            .write_line(format!("going to run {} at {}",
                cmd_str, OffsetDateTime::now_local().unwrap()))
            .await;
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
    let details = if timing && status.is_ok() {
        monitor_writer
            .write_line(format!("success: took {} ms to run {}", duration, cmd_str))
            .await;
        format!("took {} ms to run {}", duration, cmd_str)
    } else if timing && !status.is_ok() {
        eprintln!(
            "FAILED command {} in {} ms (code {})",
            cmd_str,
            duration,
            status.code()
        );
        format!("err {} in {} ms for {}", status.code(), duration, cmd_str)
    } else if !timing && !status.is_ok() {
        eprintln!("FAILED command {} (code {})", cmd_str, status.code());
        format!("err {} for {}", status.code(), cmd_str)
    } else {
        format!("finished {}", cmd_str)
    };
    debug!("{}", &details);
    if let Err(err) = sound_notification(sound_success, sound_failure, status.is_ok(), details).await {
        eprintln!("notification sound problem: {}", err);
        return ExitStatus::err();
    }
    status
}
