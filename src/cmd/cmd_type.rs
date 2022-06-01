use ::std::fmt;
use ::std::fs::File;
use ::std::fs::OpenOptions;
use ::std::fs::remove_file;
use ::std::io::BufReader;
use ::std::io::BufWriter;
use ::std::iter::Rev;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::exit;
use ::std::slice::Iter;
use ::std::slice::IterMut;

use ::log::debug;
use ::log::warn;
use ::memoize::memoize;
use ::regex::Regex;
use ::serde::Deserialize;
use ::serde::Serialize;

/// Increment for breaking changes, to avoid loading old task stack files
pub const DATA_VERSION: u32 = 2;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum ExecWith {
//     #[serde(rename = "x")]
//     Executable,
//     #[serde(rename = "b")]
//     PlainBash,
//     #[serde(rename = "p")]
//     ProfileBash,
//     #[serde(rename = "d")]
//     Docker,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    name: String,
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RunId {
    pub run_ts_s: u32,
    pub run_rand_id: u32,
    pub cmd_id: u32,
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.run_ts_s, self.run_rand_id, self.cmd_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTask {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningTask {
    #[serde(flatten)]
    pub task: PendingTask,
    pub run_id: RunId,
}

impl PendingTask {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        PendingTask { cmd, args }
    }

    pub fn new_split(parts: Vec<String>) -> Self {
        let (cmd, args) = parts.split_first().unwrap();
        PendingTask::new(cmd.to_owned(), args.to_vec())
    }

    pub fn with_run_id(self, run: RunId) -> RunningTask {
        RunningTask {
            task: self,
            run_id: run,
        }
    }

    pub fn as_cmd_str(&self) -> String {
        format!("{} {}", self.cmd, self.args.join(" "))
    }
}

impl RunningTask {
    pub fn as_cmd_str(&self) -> String {
        self.task.as_cmd_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskType {
    Pending(PendingTask),
    Running(RunningTask),
}

impl TaskType {
    pub fn is_running(&self) -> bool {
        matches!(self, TaskType::Running(_))
    }

    pub fn as_cmd_str(&self) -> String {
        match self {
            TaskType::Pending(task) => task.as_cmd_str(),
            TaskType::Running(task) => task.as_cmd_str(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStack {
    //TODO @mark: back to private
    pub(crate) tasks: Vec<TaskType>,
}

impl TaskStack {
    pub fn empty() -> Self {
        TaskStack { tasks: vec![] }
    }

    pub fn from(tasks: Vec<TaskType>) -> Self {
        TaskStack { tasks }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, TaskType> {
        self.tasks.iter_mut()
    }
}

impl TaskStack {
    pub fn add(&mut self, task: PendingTask) {
        self.tasks.push(TaskType::Pending(task));
    }

    pub fn add_running(&mut self, task: RunningTask) {
        self.tasks.push(TaskType::Running(task));
    }

    pub fn add_end(&mut self, task: PendingTask) {
        self.tasks.insert(0, TaskType::Pending(task));
    }

    pub fn pop(&mut self) -> Option<TaskType> {
        self.tasks.pop()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Iterate from next to last (reverse to `self.task`)
    pub fn iter(&self) -> Rev<Iter<TaskType>> {
        self.iter_old2new().rev()
    }

    pub fn iter_old2new(&self) -> Iter<TaskType> {
        self.tasks.iter()
    }
}
