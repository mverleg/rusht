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
use ::which::which_all;

use crate::common::{fail, LineWriter, StdWriter, Task};

#[derive(Debug)]
pub struct Dependent {
    task: Task,
    deps: SmallVec<[Mutex<()>; 2]>,
}

impl Dependent {
    pub fn new(task: Task, deps: impl Into<SmallVec<[Mutex<()>; 2]>>) -> Self {
        Dependent {
            task,
            deps: deps.into(),
        }
    }
}
