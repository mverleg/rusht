use ::std::io::Cursor;
use std::env;

use ::log::debug;
use ::log::warn;
use ::rodio::{PlayError, Decoder, OutputStream, Source};

use crate::common::{LineReader, LineWriter};
use crate::ExitStatus;
use crate::observe::mon_args::MonArgs;

pub async fn mon(
    args: MonArgs,
    _reader: &mut impl LineReader,
    _writer: &mut impl LineWriter,
) -> ExitStatus {
    let task = args.cmd.clone().into_task();
    let status = task.execute(false);
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
        //TODO @mverleg: path
    } else {
        debug!("not playing sound because not requested for success={}", is_success);
        return Ok(())
    };
    play_sound(cursor)
}

fn play_sound(sound_cursor: Cursor<&[u8]>) -> Result<(), String> {
    let decoder = Decoder::new(sound_cursor)
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
