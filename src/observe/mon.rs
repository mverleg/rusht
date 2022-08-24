use ::std::io::Cursor;
use std::env;

use ::log::debug;
use ::log::warn;
use ::rodio::{PlayError, Decoder, OutputStream, Source};
use rodio::{cpal, Devices};
use rodio::cpal::{Host, host_from_id};
use rodio::cpal::platform::CoreAudioHost;
use rodio::cpal::traits::HostTrait;
use rodio::DeviceTrait;

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
    let mut chosen_device = None;
    for host in cpal::available_hosts() {
        let host = host_from_id(host).unwrap();
        debug!("host: {:?}", host.id());
        for dev in host.output_devices().unwrap() {
            debug!("  audio device: {:?}", dev.name());
            if dev.name().unwrap() == "Razer Kraken X USB" {
                chosen_device = Some(dev);
            }
        }
    }

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

    let (_, stream_handle) = OutputStream::try_from_device(&chosen_device.unwrap()).unwrap();
    // let (_, stream_handle) = stream.or_else(|original_err| {
    //     // default device didn't work, try other ones
    //     let mut devices = match cpal::default_host().output_devices() {
    //         Ok(d) => d,
    //         Err(_) => return Err(original_err),
    //     };
    //
    //     devices
    //         .find_map(|d| Self::try_from_device(&d).ok())
    //         .ok_or(original_err)
    // })
    //    .map_err(|err| format!("failed to get default sound output device, err: {}", err))?;
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
