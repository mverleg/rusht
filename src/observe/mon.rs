use ::std::time::Instant;

use crate::common::{LineWriter, Task, VecWriter};
use crate::ExitStatus;
use crate::observe::mon_args::MonArgs;
use crate::observe::sound_notification;

pub async fn mon(
    args: MonArgs,
    writer: &mut impl LineWriter,
) -> ExitStatus {
    let task = args.cmd.clone().into_task();
    mon_task(&task,
             writer,
             !args.no_print_cmd,
             !args.no_output_on_success,
             !args.no_timing,
             args.sound_success,
             args.sound_failure).await
}

pub async fn mon_task(
    task: &Task,
    writer: &mut impl LineWriter,
    print_cmd: bool,
    output_on_success: bool,
    timing: bool,
    sound_success: bool,
    sound_failure: bool,
) -> ExitStatus {
    let cmd_str = task.as_str();
    if ! print_cmd {
        println!("going to run {}", cmd_str);
    }
    let t0 = Instant::now();
    let status = if output_on_success {
        let mut out_buffer = VecWriter::new();
        let status = task.execute_with_stdout_nomonitor(&mut out_buffer).await;
        if status.is_err() {
            eprintln!("printing all output because process failed");
            writer.write_all_lines(out_buffer.get().iter()).await;
        }
        status
    } else {
        task.execute_with_stdout_nomonitor(writer).await
    };
    let duration = t0.elapsed().as_millis();
    if ! timing {
        if status.is_ok() {
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
                status.code()
            );
        }
    }
    if let Err(err) = sound_notification(sound_success, sound_failure, status.is_ok()) {
        eprintln!("notification sound problem: {}", err);
        return ExitStatus::err()
    }
    status
}
