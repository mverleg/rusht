use ::std::collections::HashMap;
use ::std::process::exit;
use ::std::process::Command;
use ::std::process::Stdio;
use ::std::time::Instant;

use ::log::debug;
use ::rand::Rng;
use ::structopt::StructOpt;

use crate::cmd_io::current_time_s;
use crate::cmd_io::fail;
use crate::cmd_io::read;
use crate::cmd_io::write;
use crate::cmd_type::RunId;
use crate::cmd_type::RunningTask;
use crate::cmd_type::TaskStack;
use crate::cmd_type::TaskType;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmdo",
    about = "Execute a command and remove it from the stack if successful. See also cmadd, cmlist, cmdrop"
)]
pub struct DoArgs {
    #[structopt(
        short = "c",
        long,
        default_value = "1",
        help = "Number of commands to run"
    )]
    pub count: u32,
    #[structopt(
        short = "n",
        long,
        default_value = "",
        help = "Use the stack from the given namespace instead of the global one"
    )]
    pub namespace: String,
    #[structopt(
        short = "a",
        long,
        help = "Keep running commands until one fails or the stack is empty",
        conflicts_with = "count",
        conflicts_with = "parallel"
    )]
    pub autorun: bool,
    #[structopt(
        short = "p",
        long,
        help = "Whether to run commands in parallel (if more than one)"
    )]
    pub parallel: bool,
    #[structopt(
        short = "r",
        long,
        help = "Always remove the command(s) from the stack, even if they fail",
        conflicts_with = "keep"
    )]
    pub always_pop: bool,
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
        let status = execute(task, args.quiet);
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
        let run_task = pending.with_run_id(run_id);
        to_run.push(run_task.clone());
        *task = TaskType::Running(run_task);
        run_nr += 1;
        if !args.autorun && run_nr == args.count {
            break;
        }
    }
    to_run
}

fn execute(task: RunningTask, quiet: bool) -> Status {
    let t0 = Instant::now();
    let cmd_str = task.as_cmd_str();
    let mut child = match Command::new(task.task.cmd)
        .args(&task.task.args)
        //.shell(true)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => fail(format!(
            "failed to start command '{}', error {}",
            cmd_str, err
        )),
    };
    let status = match child.wait() {
        Ok(status) => status,
        Err(err) => fail(format!(
            "failed to finish command '{}', error {}",
            cmd_str, err
        )),
    };
    if !quiet {
        let duration = t0.elapsed().as_millis();
        println!("took {} ms to run: {}", duration, cmd_str);
    }
    if status.success() {
        Status::Success
    } else {
        Status::Failed
    }
}

fn remove_completed_tasks(
    args: &DoArgs,
    tasks: TaskStack,
    statuses: &HashMap<RunId, Status>,
) -> TaskStack {
    let filtered_tasks = tasks
        .iter()
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
