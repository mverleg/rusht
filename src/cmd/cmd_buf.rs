use ::clap::StructOpt;
use crate::ExitStatus;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmbuf",
    about = "Read input, build commands and buffer them, then run them all. Somewhat xargs."
)]
pub struct BufArgs {

    // #[structopt(short = 'e', long)]
    // /// Add command at the end (last) instead of as the next.
    // pub end: bool,
    #[structopt(short = 'L', long)]
    /// Give a replacement placeholder for each line, instead of '{}'.
    pub lines_with: Option<String>,
    #[structopt(short = 'u', long)]
    /// Skip any duplicate placeholders.
    pub unique: bool,
    #[structopt(short = 'P', long)]
    /// Working directory when running the command. Can use placeholder with -L.
    pub working_dir: Option<String>,

    #[structopt(short = 'c', long)]
    /// Maximum number of commands to run (others are forgotten).
    pub count: u32,
    #[structopt(short = 'p', long = "parallel", default_value = "1")]
    /// How many parallel tasks to run (implies --continue-on-error).
    pub parallel: u32,
    #[structopt(short = 'f', long = "continue-on-error")]
    /// Keep running tasks even if one fails.
    pub continue_on_error: bool,
    #[structopt(short = 'q', long)]
    /// Do not log command and timing.
    pub quiet: bool,
    // #[structopt(short = '0', long = "allow-empty")]
    // /// Silently do nothing if there are no commands.
    // pub allow_empty: bool,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    BufArgs::into_app().debug_assert()
}

pub fn buf_cmd(args: BufArgs) -> ExitStatus {
    //TODO @mverleg: lines_with
    //TODO @mverleg: unique
    //TODO @mverleg: working_dir
    //TODO @mverleg: count
    //TODO @mverleg: parallel
    //TODO @mverleg: continue_on_error
    //TODO @mverleg: quiet
    unimplemented!()  //TODO @mverleg:
}

// pub fn do_cmd(args: DoArgs) -> bool {
//     let args = verify_args(args);
//     let ts_s = current_time_s();
//     let mut tasks = read(args.namespace.clone());
//     if tasks.is_empty() {
//         if args.allow_empty {
//             return true;
//         }
//         eprintln!("there are no commands to run, use cmadd to add them");
//         return false;
//     }
//
//     let to_run = mark_tasks_to_run(&args, &mut tasks, ts_s);
//     write(args.namespace.clone(), &tasks);
//
//     let statuses = Arc::new(DashMap::new());
//     to_run
//         .iter()
//         .map(|task| task.run_id)
//         .map(|run_id| (run_id, Status::Skipped))
//         .for_each(|(id, status)| {
//             statuses.insert(id, status);
//         });
//     let total_count = to_run.len();
//     let current_nr = AtomicUsize::new(1);
//
//     if args.continue_on_error {
//         ThreadPoolBuilder::new()
//             .num_threads(args.parallel as usize)
//             .build()
//             .expect("failed to create thread pool")
//             .install(|| {
//                 to_run
//                     .into_par_iter()
//                     .map(|task| {
//                         let (id, status) = exec(
//                             &args,
//                             task,
//                             current_nr.fetch_add(1, Ordering::AcqRel),
//                             total_count,
//                         );
//                         statuses.insert(id, status);
//                     })
//                     .for_each(|_| {});
//             });
//     } else {
//         assert!(
//             args.parallel <= 1,
//             "cannot use parallel mode when continue-on-error is true"
//         );
//         to_run
//             .into_iter()
//             .map(|task| {
//                 let (id, status) = exec(
//                     &args,
//                     task,
//                     current_nr.fetch_add(1, Ordering::AcqRel),
//                     total_count,
//                 );
//                 statuses.insert(id, status);
//                 status
//             })
//             .take_while(|status| status == &Status::Success)
//             .for_each(|_| {});
//     }
//
//     let tasks = read(args.namespace.clone());
//     let remaining = remove_completed_tasks(&args, tasks, &statuses);
//     write(args.namespace, &remaining);
//
//     if !args.quiet {
//         if remaining.is_empty() {
//             println!("no commands left");
//         } else {
//             println!("{} command(s) left", remaining.len());
//         }
//     }
//     let all_ok = statuses
//         .iter()
//         .all(|entry| *entry.value() == Status::Success);
//     all_ok
// }
//
// fn verify_args(mut args: DoArgs) -> DoArgs {
//     if args.parallel > 1 && !args.continue_on_error {
//         info!("enabling --continue-on-error because of --parallel");
//         args.continue_on_error = true
//     }
//     if args.all {
//         args.count = 0 // to spot bugs
//     }
//     if args.all || args.count < args.parallel {
//         info!(
//             "--parallel ({}) is higher than --count ({})",
//             args.parallel, args.count
//         );
//     }
//     args
// }
//
// fn exec(
//     args: &DoArgs,
//     task: RunningTask,
//     current_nr: usize,
//     total_count: usize,
// ) -> (RunId, Status) {
//     if !args.quiet {
//         if total_count > 1 {
//             println!("run {}/{}: {}", current_nr, total_count, task.as_str());
//         } else {
//             println!("run: {}", task.as_str());
//         }
//     }
//     let id = task.run_id;
//     let status = Status::from(task.task.execute_sync(!args.quiet));
//     (id, status)
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum Status {
//     Success,
//     Failed,
//     Skipped,
// }
//
// impl From<ExitStatus> for Status {
//     fn from(exit: ExitStatus) -> Self {
//         if exit.is_ok() {
//             Status::Success
//         } else {
//             Status::Failed
//         }
//     }
// }
//
// fn mark_tasks_to_run(args: &DoArgs, tasks: &mut TaskStack, ts_s: u32) -> Vec<RunningTask> {
//     let rand_id = rand::thread_rng().gen::<u32>();
//     let mut run_nr = 0;
//     let mut to_run = vec![];
//     for task in tasks.iter_mut() {
//         let pending = match task {
//             TaskType::Running(running) => {
//                 debug!(
//                     "still running with run-id {}, command {}",
//                     running.run_id,
//                     running.as_str()
//                 );
//                 if args.restart_running {
//                     running.task.clone()
//                 } else {
//                     eprintln!("skipping command because it is already running or has failed without contact: {}",
//                               running.as_str());
//                     continue;
//                 }
//             }
//             TaskType::Pending(task) => task.clone(),
//         };
//         let run_id = RunId {
//             run_ts_s: ts_s,
//             run_rand_id: rand_id,
//             cmd_id: run_nr,
//         };
//         debug!(
//             "assigning run-id {} to command {}",
//             run_id,
//             pending.as_str()
//         );
//         let run_task = RunningTask::new(pending, run_id);
//         to_run.push(run_task.clone());
//         *task = TaskType::Running(run_task);
//         run_nr += 1;
//         if !args.all && run_nr >= args.count {
//             break;
//         }
//     }
//     to_run
// }
//
// fn remove_completed_tasks(
//     args: &DoArgs,
//     tasks: TaskStack,
//     statuses: &DashMap<RunId, Status>,
// ) -> TaskStack {
//     let filtered_tasks = tasks
//         .iter_old2new()
//         .flat_map(|task| should_keep_completed_task(task, args, statuses))
//         .collect();
//     TaskStack::from(filtered_tasks)
// }
//
// fn should_keep_completed_task(
//     task: &TaskType,
//     args: &DoArgs,
//     statuses: &DashMap<RunId, Status>,
// ) -> Option<TaskType> {
//     let cmd = task.as_cmd_str();
//     match task {
//         TaskType::Pending(pending) => {
//             debug!("keep command because it is not running: {}", &cmd);
//             Some(TaskType::Pending(pending.clone()))
//         }
//         TaskType::Running(running) => match statuses.get(&running.run_id) {
//             Some(value_ref) => match value_ref.value() {
//                 Status::Success => {
//                     if args.keep_successful {
//                         debug!("keep successful command because all tasks kept: {}", &cmd);
//                         Some(TaskType::Running(running.clone()))
//                     } else {
//                         debug!("removing successful command: {}", &cmd);
//                         None
//                     }
//                 }
//                 Status::Failed => {
//                     if args.drop_failed {
//                         debug!(
//                             "removing failed command (as requested with --drop-failed): {}",
//                             &cmd
//                         );
//                         None
//                     } else {
//                         debug!("keep failed command to be retried: {}", &cmd);
//                         Some(TaskType::Running(running.clone()))
//                     }
//                 }
//                 Status::Skipped => {
//                     debug!("keep skipped command to be retried: {}", &cmd);
//                     Some(TaskType::Running(running.clone()))
//                 }
//             },
//             None => {
//                 eprintln!(
//                     "command is running but not started by current run: {}",
//                     &cmd
//                 );
//                 Some(TaskType::Running(running.clone()))
//             }
//         },
//     }
// }
