use ::std::env;

use ::log::debug;
use ::log::error;
use futures::future::join;
use crate::common::StdWriter;
use crate::common::Task;

//TODO @mverleg: rename since not just sound anymore?
pub async fn sound_notification(
    sound_on_success: bool,
    sound_on_failure: bool,
    is_success: bool,
    details: String,
) -> Result<(), String> {
    if ! env::var("NO_SOUND").unwrap_or("".to_owned()).trim().is_empty() {
        debug!("sound suppressed by NO_SOUND env var");
        return Ok(())
    }
    let popup_msg = format!("display notification \"{}\" with title \"{} (mon)\"",
            details.replace("\"", "").replace("'", "").replace("\\", "\\\\"),
            if is_success { "OK" } else { "FAILED"});
    let (sound_task, popup_task) = if is_success && sound_on_success {
        if !env::var("MON_NESTED_SOUND_OK").unwrap_or("".to_owned()).trim().is_empty() {
            debug!("success sound suppressed because of nested `mon` invocations; parent should already play sound");
            return Ok(())
        }
        (
            Task::new_in_cwd("say".to_owned(), None, vec!["ready".to_owned()]),
            Task::new_in_cwd("osascript".to_owned(), None, vec!["-e".to_owned(), popup_msg])
        )
    } else if !is_success && sound_on_failure {
        if !env::var("MON_NESTED_SOUND_ERR").unwrap_or("".to_owned()).trim().is_empty() {
            debug!("error sound suppressed because of nested `mon` invocations; parent should already play sound");
            return Ok(())
        }
        (
            Task::new_in_cwd("say".to_owned(), None, vec!["that failed".to_owned()]),
            Task::new_in_cwd("osascript".to_owned(), None, vec!["-e".to_owned(), popup_msg])
        )
    } else {
        return Ok(())
    };
    //TODO @mverleg: use block_on since async wants recursive future type, and we anyway want to wait
    let (sound_status, popup_status) = join(
        sound_task.execute_with_stdout_nomonitor(
            &mut StdWriter::stdout(),
            &mut StdWriter::stderr()
        ),
        popup_task.execute_with_stdout_nomonitor(
            &mut StdWriter::stdout(),
            &mut StdWriter::stderr()
        ),
    ).await;
    if sound_status.is_err() {
        return Err(format!("failed to play sound using {}", &sound_task.as_cmd_str()))
    }
    if popup_status.is_err() {
        error!("failed to show popup using {}", &popup_task.as_cmd_str())
    }
    Ok(())

    // let sl = Soloud::default().unwrap();
    // let mut sound = audio::Wav::default();
    // sound.load_mem(include_bytes!("../../resource/success-sound.mp3")).unwrap();
    // sl.play(&sound)
    // let cursor = if is_success && sound_on_success {
    //     Cursor::new(include_bytes!("../../resource/success-sound.mp3").as_ref())
    // } else if !is_success && sound_on_failure {
    //     Cursor::new(include_bytes!("../../resource/error-sound.mp3").as_ref())
    // } else {
    //     debug!(
    //         "not playing sound because not requested for success={}",
    //         is_success
    //     );
    //     return Ok(());
    // };
    // //TODO @mverleg: for some reason I cannot extract below into a function because it complains about lifetime of cursor
    // let decoder =
    //     Decoder::new(cursor).map_err(|err| format!("failed to decode sound, err: {}", err))?;
    // let (_, stream_handle) = OutputStream::try_default()
    //     .map_err(|err| format!("failed to get default sound output device, err: {}", err))?;
    // match stream_handle.play_raw(decoder.convert_samples()) {
    //     Ok(()) => Ok(()),
    //     Err(err) => match err {
    //         PlayError::DecoderError(err) => Err(format!(
    //             "failed to play notification sound because it could not be decoded, err: {}",
    //             err
    //         )),
    //         PlayError::NoDevice => {
    //             let debug_env_name = "MON_SUPPRESS_AUDIO_DEVICE_WARNING";
    //             if env::var(debug_env_name).is_err() {
    //                 warn!("could not play notification sound because no default output device was found");
    //                 debug!(
    //                     "no default audio device; set env {} to suppress warning",
    //                     debug_env_name
    //                 );
    //             }
    //             Ok(())
    //         }
    //     },
    // }
}
