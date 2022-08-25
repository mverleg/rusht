use ::std::collections::HashMap;
use ::std::env;
use ::std::io::{BufRead, BufReader};
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::ExitStatus as ProcStatus;
use ::std::process::Stdio;
use ::std::time::Instant;

use ::async_std::sync::Mutex;
use ::async_std::task::block_on;
use ::clap::StructOpt;
use ::dashmap::DashMap;
use ::itertools::Itertools;
use ::lazy_static::lazy_static;
use ::log::{debug, warn};
use ::log::info;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::smallvec::SmallVec;
use ::time::Duration;
use ::which::which_all;

use crate::common::{fail, LineWriter, StdWriter, Task};

#[derive(Debug)]
pub struct Dependency {
    name: String,
    lock: Mutex<()>,
    timeout: Duration,
}

impl Dependency {
    pub fn new_unlimited(name: String, lock: Mutex<()>) -> Self {
        Dependency::new_timeout(name, lock, Duration::new(i64::MAX, 0))
    }

    pub fn new_timeout(name: String, lock: Mutex<()>, timeout: Duration) -> Self {
        Dependency {
            name,
            lock,
            timeout,
        }
    }
}

#[derive(Debug)]
pub struct Dependent {
    task: Task,
    current: Mutex<()>,
    dependencies: SmallVec<[Dependency; 1]>,
}

impl Dependent {
    pub fn new(task: Task, current: Mutex<()>, dependencies: impl Into<SmallVec<[Dependency; 1]>>) -> Self {
        Dependent {
            task,
            current: Default::default(),
            dependencies: dependencies.into(),
        }
    }
}
