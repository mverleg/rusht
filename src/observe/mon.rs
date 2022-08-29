use ::std::env;
use ::std::io::Cursor;
use ::std::time::Instant;

use ::log::debug;
use ::log::warn;
use ::rodio::{Decoder, OutputStream, PlayError, Source};

use crate::common::{LineReader, LineWriter, StdWriter, VecWriter};
use crate::ExitStatus;
use crate::observe::mon_args::MonArgs;

pub async fn mon(
    args: MonArgs,
    _reader: &mut impl LineReader,
    _writer: &mut impl LineWriter,
) -> ExitStatus {
    let task = args.cmd.clone().into_task();
    let mut out_writer = StdWriter::stdout();
    let t0 = Instant::now();
    let cmd_str = task.as_str();
    if ! args.no_print_cmd {
        println!("going to run {}", cmd_str);
    }
    let t0 = Instant::now();
    let status = if args.no_output_on_success {
        let mut out_buffer = VecWriter::new();
        let status = task.execute_with_stdout(true, &mut out_buffer).await;
        if !status.success() {
            eprintln!("printing all output because process failed");
            out_writer.write_all_lines(out_buffer.get().iter()).await;
        }
        status
    } else {
        task.execute_with_stdout(false, &mut out_writer).await
    };
    let duration = t0.elapsed().as_millis();
    if ! args.no_timing {
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
    if let Err(err) = sound_notification(&args, status.success()) {
        eprintln!("notification sound problem: {}", err);
        return ExitStatus::err()
    }
    ExitStatus::of_code(status.code())
    //TODO @mverleg:
}

fn sound_notification(args: &MonArgs, is_success: bool) -> Result<(), String> {
    let cursor = if is_success && args.sound_success {
        Cursor::new(include_bytes!("../../resource/success-sound.mp3").as_ref())
    } else if !is_success && args.sound_failure {
        Cursor::new(include_bytes!("../../resource/error-sound.mp3").as_ref())
    } else {
        debug!("not playing sound because not requested for success={}", is_success);
        return Ok(())
    };
    //TODO @mverleg: for some reason I cannot extract below into a function because it complains about lifetime of cursor
    let decoder = Decoder::new(cursor)
        .map_err(|err| format!("failed to decode sound, err: {}", err))?;
    let (_, stream_handle) = OutputStream::try_default()
        .map_err(|err| format!("failed to get default sound output device, err: {}", err))?;
    match stream_handle.play_raw(decoder.convert_samples()) {
        Ok(()) => Ok(()),
        Err(err) => match err {
            PlayError::DecoderError(err) => Err(format!("failed to play notification sound because it could not be decoded, err: {}", err)),
            PlayError::NoDevice => {
                let debug_env_name = "MON_SUPPRESS_AUDIO_DEVICE_WARNING";
                if !env::var(debug_env_name).is_ok() {
                    warn!("could not play notification sound because no default output device was found");
                    debug!("no default audio device; set env {} to suppress warning", debug_env_name);
                }
                Ok(())
            }
        }
    }
}
