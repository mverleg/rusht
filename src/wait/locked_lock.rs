use ::chrono::{DateTime, Local};
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::Task;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LockFile {
    key: String,
    writer: Option<LockHolder>,
    readers: Vec<LockHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LockHolder {
    pid: u8,
    acquired: DateTime<Local>,
    task: Task,
}
