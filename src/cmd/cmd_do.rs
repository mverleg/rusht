use ::std::collections::HashMap;
use ::std::process::ExitStatus;

use ::log::debug;
use ::rand::Rng;
use ::structopt::StructOpt;

use crate::cmd::cmd_io::current_time_s;
use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::write;
use crate::cmd::cmd_type::RunId;
use crate::cmd::cmd_type::RunningTask;
use crate::cmd::cmd_type::TaskStack;
use crate::cmd::cmd_type::TaskType;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmdo",
    about = "Execute a command and remove it from the stack if successful. See also cmadd, cmlist, cmdrop"
)]
pub struct DoArgs {
    #[structopt(
        short = "n",
        long,
        default_value = "",
        help = "Use the stack from the given namespace instead of the global one"
    )]
    pub namespace: String,
    #[structopt(
        short = "c",
        long,
        default_value = "1",
        help = "Number of commands to run"
    )]
    pub count: u32,
    #[structopt(
        short = "a",
        long,
        help = "Try to run all the commands",
        conflicts_with = "count"
    )]
    pub all: bool,
    #[structopt(
        short = "p",
        long,
        default_value = "1",
        help = "How many tasks to run in parallel at any time"
    )]
    pub parallel: u32,
    #[structopt(
        short = "x",
        long = "on-err",
        help = "What to do when a command fails: [a]bort, [c]ontinue but keep on stack, or continue and [d]rop it",
        conflicts_with = "keep"
    )]
    pub on_err: OnErr,
    #[structopt(
        short = "k",
        long,
        help = "Execute the command but keep it on the stack",
        conflicts_with = "always_pop"
    )]
    pub keep: bool,
    #[structopt(short = "q", long, help = "Do not log command and timing")]
    pub quiet: bool,
}

#[derive(StructOpt, Debug, Clone, Default)]
pub enum OnErr {
    #[default]
    Abort,
    KeepContinue,
    DropContinue,
}

pub fn do_cmd(args: DoArgs) -> bool {
    let ts_s = current_time_s();
    let mut tasks = read(args.namespace.clone());
    if tasks.is_empty() {
        eprintln!("there are no commands to run, use cmadd to add them");
        return false;
    }

    let to_run = mark_tasks_to_run(&args, &mut tasks, ts_s);
    write(args.namespace.clone(), &tasks);

    let mut statuses = to_run
        .iter()
        .map(|task| task.run_id)
        .map(|run_id| (run_id, Status::Skipped))
        .collect::<HashMap<_, _>>();
    for task in to_run {
        if !args.quiet {
            println!("run: {}", task.as_cmd_str());
        }
        let run_id = task.run_id;
        let status = task.task.execute(args.quiet);
        let status = Status::from(status);
        statuses.insert(run_id, status);
        if status != Status::Success {
            break;
        }
    }

    let tasks = read(args.namespace.clone());
    let remaining = remove_completed_tasks(&args, tasks, &statuses);
    write(args.namespace, &remaining);

    if !args.quiet {
        if remaining.is_empty() {
            println!("no commands left");
        } else {
            println!("{} command(s) left", remaining.len());
        }
    }
    statuses.values().all(|status| status == &Status::Success)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Success,
    Failed,
    Skipped,
}

impl From<ExitStatus> for Status {
    fn from(exit: ExitStatus) -> Self {
        if exit.success() {
            Status::Success
        } else {
            Status::Failed
        }
    }
}

fn mark_tasks_to_run(args: &DoArgs, tasks: &mut TaskStack, ts_s: u32) -> Vec<RunningTask> {
    let rand_id = rand::thread_rng().gen::<u32>();
    let mut run_nr = 0;
    let mut to_run = vec![];
    for task in tasks.iter_mut() {
        let pending = match task {
            TaskType::Running(running) => {
                debug!(
                    "still running with run-id {}, command {}",
                    running.run_id,
                    running.as_cmd_str()
                );
                eprintln!("skipping command because it is already running or has failed without contact: {}",
                          running.as_cmd_str());
                continue;
            }
            TaskType::Pending(task) => task.clone(),
        };
        let run_id = RunId {
            run_ts_s: ts_s,
            run_rand_id: rand_id,
            cmd_id: run_nr,
        };
        debug!(
            "assigning run-id {} to command {}",
            run_id,
            pending.as_cmd_str()
        );
        let run_task = RunningTask::new(pending, run_id);
        to_run.push(run_task.clone());
        *task = TaskType::Running(run_task);
        run_nr += 1;
        if !args.autorun && run_nr == args.count {
            break;
        }
    }
    to_run
}

fn remove_completed_tasks(
    args: &DoArgs,
    tasks: TaskStack,
    statuses: &HashMap<RunId, Status>,
) -> TaskStack {
    let filtered_tasks = tasks
        .iter_old2new()
        .flat_map(|task| should_keep_completed_task(task, args, statuses))
        .collect();
    TaskStack::from(filtered_tasks)
}

fn should_keep_completed_task(
    task: &TaskType,
    args: &DoArgs,
    statuses: &HashMap<RunId, Status>,
) -> Option<TaskType> {
    let cmd = task.as_cmd_str();
    match task {
        TaskType::Pending(pending) => {
            debug!("keep command because it is not running: {}", &cmd);
            Some(TaskType::Pending(pending.clone()))
        }
        TaskType::Running(running) => match statuses.get(&running.run_id) {
            Some(Status::Success) => {
                if args.keep {
                    debug!("keep successful command because all tasks kept: {}", &cmd);
                    Some(TaskType::Running(running.clone()))
                } else {
                    debug!("removing successful command: {}", &cmd);
                    None
                }
            }
            Some(Status::Failed) => {
                if args.always_pop {
                    debug!(
                        "removing failed command because all started tasks are removed: {}",
                        &cmd
                    );
                    None
                } else {
                    debug!("keep failed command to be retried: {}", &cmd);
                    Some(TaskType::Running(running.clone()))
                }
            }
            Some(Status::Skipped) => {
                debug!("keep skipped command to be retried: {}", &cmd);
                Some(TaskType::Running(running.clone()))
            }
            None => {
                eprintln!(
                    "command is running but not started by current run: {}",
                    &cmd
                );
                Some(TaskType::Running(running.clone()))
            }
        },
    }
}
