use chrono::{DateTime, Local};
use crate::common::Task;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LockFile {
    writer: Option<LockHolder>,
    readers: Vec<LockHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LockHolder {
    pid: Process,
    acquired: DateTime<Local>,
    task: Task,
}
