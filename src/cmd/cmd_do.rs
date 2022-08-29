use ::std::sync::atomic::AtomicUsize;
use ::std::sync::atomic::Ordering;
use ::std::sync::Arc;

use ::clap::StructOpt;
use ::dashmap::DashMap;
use ::log::debug;
use ::log::info;
use ::rand::Rng;
use ::rayon::iter::{IntoParallelIterator, ParallelIterator};
use ::rayon::ThreadPoolBuilder;

use crate::cmd::cmd_io::current_time_s;
use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::write;
use crate::cmd::cmd_type::RunId;
use crate::cmd::cmd_type::RunningTask;
use crate::cmd::cmd_type::TaskStack;
use crate::cmd::cmd_type::TaskType;
use crate::ExitStatus;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmdo",
    about = "Execute a command and remove it from the stack if successful. See also cmadd, cmlist, cmdrop"
)]
pub struct DoArgs {
    #[structopt(short = 'n', long, default_value = "")]
    /// Use the stack from the given namespace instead of the global one.
    pub namespace: String,
    #[structopt(short = 'c', long, default_value = "1")]
    /// Number of commands to run.
    pub count: u32,
    #[structopt(short = 'a', long, conflicts_with = "count")]
    /// Try to run all the commands.
    pub all: bool,
    #[structopt(short = 'p', long = "parallel", default_value = "1")]
    /// How many parallel tasks to run (implies --continue-on-error).
    pub parallel: u32,
    #[structopt(short = 'g', long = "restart-running")]
    /// Run tasks even if they are marked as already running.
    pub restart_running: bool,
    #[structopt(short = 'f', long = "continue-on-error")]
    /// Keep running tasks even if one fails (it stays on stack unless -r).
    pub continue_on_error: bool,
    #[structopt(short = 'r', long = "drop-failed")]
    /// Remove tasks from the stack when ran, even if they fail.
    pub drop_failed: bool,
    #[structopt(short = 'k', long = "keep")]
    /// Keep the task on the stack when ran when successful.
    pub keep_successful: bool,
    #[structopt(short = 'q', long)]
    /// Do not log command and timing.
    pub quiet: bool,
    #[structopt(short = '0', long = "allow-empty")]
    /// Silently do nothing if there are no commands.
    pub allow_empty: bool,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    DoArgs::into_app().debug_assert()
}

pub fn do_cmd(args: DoArgs) -> bool {
    let args = verify_args(args);
    let ts_s = current_time_s();
    let mut tasks = read(args.namespace.clone());
    if tasks.is_empty() {
        if args.allow_empty {
            return true;
        }
        eprintln!("there are no commands to run, use cmadd to add them");
        return false;
    }

    let to_run = mark_tasks_to_run(args.restart_running, args.all, args.count, &mut tasks, ts_s);
    write(args.namespace.clone(), &tasks);

    let statuses = run_tasks(to_run, args.continue_on_error, args.parallel, args.quiet);

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
    let all_ok = statuses
        .iter()
        .all(|entry| *entry.value() == Status::Success);
    all_ok
}

pub fn run_tasks(
    to_run: Vec<RunningTask>,
    continue_on_error: bool,
    parallel: u32,
    quiet: bool,
) -> Arc<DashMap<RunId, Status>> {
    let statuses = Arc::new(DashMap::new());
    to_run
        .iter()
        .map(|task| task.run_id)
        .map(|run_id| (run_id, Status::Skipped))
        .for_each(|(id, status)| {
            statuses.insert(id, status);
        });
    let total_count = to_run.len();
    let current_nr = AtomicUsize::new(1);

    if continue_on_error {
        ThreadPoolBuilder::new()
            .num_threads(parallel as usize)
            .build()
            .expect("failed to create thread pool")
            .install(|| {
                to_run
                    .into_par_iter()
                    .map(|task| {
                        let (id, status) = exec(
                            task,
                            current_nr.fetch_add(1, Ordering::AcqRel),
                            total_count,
                            quiet,
                        );
                        statuses.insert(id, status);
                    })
                    .for_each(|_| {});
            });
    } else {
        assert!(
            parallel <= 1,
            "cannot use parallel mode when continue-on-error is true"
        );
        to_run
            .into_iter()
            .map(|task| {
                let (id, status) = exec(
                    task,
                    current_nr.fetch_add(1, Ordering::AcqRel),
                    total_count,
                    quiet,
                );
                statuses.insert(id, status);
                status
            })
            .take_while(|status| status == &Status::Success)
            .for_each(|_| {});
    }
    statuses
}

fn verify_args(mut args: DoArgs) -> DoArgs {
    if args.parallel > 1 && !args.continue_on_error {
        info!("enabling --continue-on-error because of --parallel");
        args.continue_on_error = true
    }
    if args.all {
        args.count = 0 // to spot bugs
    }
    if args.all || args.count < args.parallel {
        info!(
            "--parallel ({}) is higher than --count ({})",
            args.parallel, args.count
        );
    }
    args
}

fn exec(task: RunningTask, current_nr: usize, total_count: usize, quiet: bool) -> (RunId, Status) {
    if !quiet {
        if total_count > 1 {
            println!("run {}/{}: {}", current_nr, total_count, task.as_str());
        } else {
            println!("run: {}", task.as_str());
        }
    }
    let id = task.run_id;
    let status = Status::from(task.task.execute_sync(!quiet));
    (id, status)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Failed(ExitStatus),
    Skipped,
}

impl Status {
    pub fn exit_status(&self) -> ExitStatus {
        match self {
            Status::Success => ExitStatus::ok(),
            Status::Skipped => ExitStatus::ok(),
            Status::Failed(code) => ExitStatus::of(code.code),
        }
    }
}

impl From<ExitStatus> for Status {
    fn from(exit: ExitStatus) -> Self {
        if exit.is_ok() {
            Status::Success
        } else {
            Status::Failed(exit)
        }
    }
}

pub fn mark_tasks_to_run(
    restart_running: bool,
    all: bool,
    count: u32,
    tasks: &mut TaskStack,
    ts_s: u32,
) -> Vec<RunningTask> {
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
                if restart_running {
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
        if !all && run_nr >= count {
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
                Status::Failed(code) => {
                    if args.drop_failed {
                        debug!(
                            "removing failed command (as requested with --drop-failed): {} (code {})",
                            &cmd, code
                        );
                        None
                    } else {
                        debug!(
                            "keep failed command to be retried: {} (code {})",
                            &cmd, code
                        );
                        Some(TaskType::Running(running.clone()))
                    }
                }
                Status::Skipped => {
                    debug!("keep skipped command to be retried: {}", &cmd);
                    Some(TaskType::Running(running.clone()))
                }
            },
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
