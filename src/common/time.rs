use ::time::OffsetDateTime;
use ::time::macros::format_description;

pub fn current_time_user_str() -> String {
    // the `time` crate always fails on Mac and Linux when inferring timezone
    // because  of a vulnerability in libc when getting timezones
    let format = format_description!("[hour]:[minute]:[second]");
    match OffsetDateTime::now_local() {
        Ok(time) => time.format(format).unwrap(),
        Err(_) => format!("{} (utc)", OffsetDateTime::now_utc().format(format).unwrap())
    }
}
