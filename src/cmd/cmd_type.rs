use ::std::fmt;

use ::std::iter::Rev;

use ::std::slice::Iter;
use ::std::slice::IterMut;

use ::serde::Deserialize;
use ::serde::Serialize;

use crate::common::Task;

/// Increment for breaking changes, to avoid loading old task stack files
pub const DATA_VERSION: u32 = 3;

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
pub struct RunningTask {
    #[serde(flatten)]
    pub task: Task,
    pub run_id: RunId,
}

impl RunningTask {
    pub fn new(task: Task, run_id: RunId) -> Self {
        RunningTask { task, run_id }
    }

    pub fn as_cmd_str(&self) -> String {
        self.task.as_cmd_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskType {
    Pending(Task),
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
    tasks: Vec<TaskType>,
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
    pub fn add(&mut self, task: Task) {
        self.tasks.push(TaskType::Pending(task));
    }

    pub fn add_end(&mut self, task: Task) {
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
