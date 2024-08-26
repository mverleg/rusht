use ::std::time::Instant;

use ::log::debug;

use crate::common::current_time_user_str;
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
        task,
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
    mut task: &Task,
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
            .write_line(format!("{}: going to run {}",
                current_time_user_str(), cmd_str))
            .await;
    }
    let mut owned_task = None;
    if sound_success || sound_failure {
        let mut env_overrides = Vec::with_capacity(2);
        if sound_success {
            env_overrides.push(("MON_NESTED_SOUND_OK".to_owned(), cmd_str.to_owned()));
        }
        if sound_failure {
            env_overrides.push(("MON_NESTED_SOUND_ERR".to_owned(), cmd_str.to_owned()));
        }
        owned_task = Some(task.clone().with_extra_env(&env_overrides));
        task = owned_task.as_ref().unwrap();
        // ^ not beautiful, but making task owned or mutable was too impactful; this is good enough
    };
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
    let duration_ms = t0.elapsed().as_millis();
    let duration_fmtd = if duration_ms > 120_000 {
        format!("{} ms ({} min)", duration_ms, (duration_ms as f64 / 60_000.0).round() as u64)
    } else if duration_ms > 10_000 {
        format!("{} ms ({} s)", duration_ms, (duration_ms as f64 / 1000.0).round() as u64)
    } else {
        format!("{} ms", duration_ms)
    };
    let time_fmtd = current_time_user_str();
    let details = if timing && status.is_ok() {
        monitor_writer
            .write_line(format!("{} success: took {} to run {}",
                time_fmtd, duration_fmtd, cmd_str))
            .await;
        format!("took {} to run {}", duration_fmtd, cmd_str)
    } else if timing && !status.is_ok() {
        eprintln!(
            "{} FAILED command {} in {} (code {})",
            time_fmtd, cmd_str, duration_fmtd, status.code()
        );
        format!("err {} in {} for {}", status.code(), duration_fmtd, cmd_str)
    } else if !timing && !status.is_ok() {
        eprintln!("{} FAILED command {} (code {})",
            time_fmtd, cmd_str, status.code());
        format!("err {} for {}", status.code(), cmd_str)
    } else {
        format!("{} finished {}",
            time_fmtd, cmd_str)
    };
    debug!("{}", &details);
    if let Err(err) = sound_notification(sound_success, sound_failure, status.is_ok(), details).await {
        eprintln!("notification sound problem: {}", err);
        return ExitStatus::err();
    }
    status
}
