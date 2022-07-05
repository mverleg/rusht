use ::std::fs::create_dir_all;
use ::std::fs::File;
use ::std::fs::OpenOptions;
use ::std::fs::remove_file;
use ::std::io::BufReader;
use ::std::io::BufWriter;
use ::std::io::Write;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::SystemTime;
use ::std::time::UNIX_EPOCH;
use std::fmt;
use std::fmt::Formatter;

use ::chrono::{DateTime, Local};
use ::log::debug;
use ::memoize::memoize;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::fail;
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
