use ::std::process::ExitStatus;
use ::std::sync::Arc;

use ::dashmap::DashMap;
use ::log::debug;
use ::log::info;
use ::rand::Rng;
use ::rayon::iter::{IntoParallelIterator, ParallelIterator};
use ::rayon::ThreadPoolBuilder;
use ::clap::StructOpt;

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
        long = "parallel",
        default_value = "1",
        help = "How many parallel tasks to run (implies --continue-on-error)"
    )]
    pub parallel: u32,
    #[structopt(
        short = "g",
        long = "restart-running",
        help = "Run tasks even if they are marked as already running."
    )]
    pub restart_running: bool,
    #[structopt(
        short = "f",
        long = "continue-on-error",
        help = "Keep running tasks even if one fails (it stays on stack unless -r)"
    )]
    pub continue_on_error: bool,
    #[structopt(
        short = "r",
        long = "drop-failed",
        help = "Remove tasks from the stack when ran, even if they fail",
    )]
    pub drop_failed: bool,
    #[structopt(
        short = "k",
        long = "keep",
        help = "Keep the task on the stack when ran when successful",
    )]
    pub keep_successful: bool,
    #[structopt(short = "q", long, help = "Do not log command and timing")]
    pub quiet: bool,
}

pub fn do_cmd(args: DoArgs) -> bool {
    let args = verify_args(args);
    let ts_s = current_time_s();
    let mut tasks = read(args.namespace.clone());
    if tasks.is_empty() {
        eprintln!("there are no commands to run, use cmadd to add them");
        return false;
    }

    let to_run = mark_tasks_to_run(&args, &mut tasks, ts_s);
    write(args.namespace.clone(), &tasks);

    let statuses = Arc::new(DashMap::new());
    to_run.iter()
        .map(|task| task.run_id)
        .map(|run_id| (run_id, Status::Skipped))
        .for_each(|(id, status)| { statuses.insert(id, status); });

    if args.continue_on_error {
        ThreadPoolBuilder::new().num_threads(args.parallel as usize)
            .build().expect("failed to create thread pool")
            .install(|| {
                to_run.into_par_iter()
                    .map(|task| {
                        let (id, status) = exec(&args, task);
                        statuses.insert(id, status);
                    })
                    .for_each(|_| {});
            });
    } else {
        assert!(args.parallel <= 1, "cannot use parallel mode when continue-on-error is true");
        to_run.into_iter()
            .map(|task| {
                let (id, status) = exec(&args, task);
                statuses.insert(id, status);
                status
            })
            .take_while(|status| status == &Status::Success)
            .for_each(|_| {});
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
    let all_ok = statuses.iter().all(|entry| *entry.value() == Status::Success);
    all_ok
}

fn verify_args(mut args: DoArgs) -> DoArgs {
    if args.parallel > 1 && !args.continue_on_error {
        info!("enabling --continue-on-error because of --parallel");
        args.continue_on_error = true
    }
    if args.all {
        args.count = 0  // to spot bugs
    }
    if args.all || args.count < args.parallel {
        info!("--parallel ({}) is higher than --count ({})", args.parallel, args.count);
    }
    args
}

fn exec(args: &DoArgs, task: RunningTask) -> (RunId, Status) {
    if !args.quiet {
        println!("run: {}", task.as_str());
    }
    let id = task.run_id;
    let status = Status::from(task.task.execute(args.quiet));
    (id, status)
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
                    running.as_str()
                );
                if args.restart_running {
                    running.task.clone()
                } else {
                    eprintln!("skipping command because it is already running or has failed without contact: {}",
                              running.as_str());
                    continue;
                }
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
            pending.as_str()
        );
        let run_task = RunningTask::new(pending, run_id);
        to_run.push(run_task.clone());
        *task = TaskType::Running(run_task);
        run_nr += 1;
        if !args.all && run_nr >= args.count {
            break;
        }
    }
    to_run
}

fn remove_completed_tasks(
    args: &DoArgs,
    tasks: TaskStack,
    statuses: &DashMap<RunId, Status>,
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
    statuses: &DashMap<RunId, Status>,
) -> Option<TaskType> {
    let cmd = task.as_cmd_str();
    match task {
        TaskType::Pending(pending) => {
            debug!("keep command because it is not running: {}", &cmd);
            Some(TaskType::Pending(pending.clone()))
        }
        TaskType::Running(running) => match statuses.get(&running.run_id) {
            Some(value_ref) => match value_ref.value() {
                Status::Success => {
                    if args.keep_successful {
                        debug!("keep successful command because all tasks kept: {}", &cmd);
                        Some(TaskType::Running(running.clone()))
                    } else {
                        debug!("removing successful command: {}", &cmd);
                        None
                    }
                }
                Status::Failed => {
                    if args.drop_failed {
                        debug!(
                        "removing failed command (as requested with --drop-failed): {}",
                        &cmd
                    );
                        None
                    } else {
                        debug!("keep failed command to be retried: {}", &cmd);
                        Some(TaskType::Running(running.clone()))
                    }
                }
                Status::Skipped => {
                    debug!("keep skipped command to be retried: {}", &cmd);
                    Some(TaskType::Running(running.clone()))
                }
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
