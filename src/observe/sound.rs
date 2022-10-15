#![allow(unused)] //TODO @mark: TEMPORARY! REMOVE THIS!

use ::log::debug;
use ::log::warn;

pub fn sound_notification(
    sound_on_success: bool,
    sound_on_failure: bool,
    is_success: bool,
) -> Result<(), String> {
    debug!("sound temporarily disabled, awaiting fixes"); //TODO @mark: TEMPORARY! REMOVE THIS!
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
