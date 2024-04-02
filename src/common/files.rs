use ::std::time::UNIX_EPOCH;

use ::async_std::fs;

pub async fn file_modified_time_in_seconds(path: &str) -> Option<u64> {
    match fs::metadata(path).await {
        Ok(meta) => Some(meta
            .modified()
            .unwrap_or_else(|_| panic!("could not get modified time for {path}"))
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| panic!("could not get time difference for {path}"))
            .as_secs()),
        Err(_) => None
    }
}
