use ::time::OffsetDateTime;
use ::time::macros::format_description;

pub fn current_time_user_str() -> String {
    // "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
    let format = format_description!("[hour]:[minute]:[second]");
    match OffsetDateTime::now_local() {
        Ok(time) => time.format(format).unwrap(),
        Err(_) => format!("{} (utc)", OffsetDateTime::now_utc().format(format).unwrap())
    }
}
