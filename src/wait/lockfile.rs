use ::std::fmt;
use ::std::fmt::Formatter;

use ::chrono::{DateTime, Local};
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::Task;

/// Increment for breaking changes, to avoid loading old lock files
pub const DATA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Key {
    value: String,
}

impl Key {
    pub fn new(text: impl Into<String>) -> Self {
        let value = text.into();
        assert!(!value.is_empty());
        Key {
            value
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    key: Key,
    users: Users,
    readers: Vec<LockHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Users {
    Writer(LockHolder),
    Readers(Vec<LockHolder>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockHolder {
    pid: u16,
    acquired: DateTime<Local>,
    task: Task,
}
