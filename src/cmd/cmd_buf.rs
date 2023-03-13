use ::clap::Parser;

use crate::cmd::cmd_do::{mark_tasks_to_run, run_tasks};
use crate::cmd::cmd_type::{TaskStack, TaskType};
use crate::cmd::create_cmd::create_tasks;
use crate::common::{CommandArgs, EmptyLineHandling, stdin_lines};
use crate::ExitStatus;

#[derive(Parser, Debug)]
#[command(
    name = "cmbuf",
    about = "Read input, build commands and buffer them, then run them all. Somewhat like xargs."
)]
pub struct BufArgs {
    // #[structopt(short = 'e', long)]
    // /// Add command at the end (last) instead of as the next.
    // pub end: bool,
    #[arg(short = 'L', long)]
    /// Give a replacement placeholder for each line, instead of '{}'.
    pub lines_with: Option<String>,
    #[arg(short = 'u', long)]
    /// Skip any duplicate placeholders.
    pub unique: bool,
    #[arg(short = 'i', long)]
    /// Stdin to be sent to the command. Can use placeholder with -l/-L.
    pub stdin: Option<String>,
    #[arg(short = 'P', long)]
    /// Working directory when running the command. Can use placeholder with -L.
    pub working_dir: Option<String>,

    #[arg(short = 'c', long)]
    /// Maximum number of commands to run (others are forgotten).
    pub count: Option<u32>,
    #[arg(short = 'p', long = "parallel", default_value = "1")]
    /// How many parallel tasks to run (implies --continue-on-error).
    pub parallel: u32,
    #[arg(short = 'f', long = "continue-on-error")]
    /// Keep running tasks even if one fails.
    pub continue_on_error: bool,
    #[arg(short = 'q', long)]
    /// Do not log command and timing.
    pub quiet: bool,
    // #[structopt(short = '0', long = "allow-empty")]
    // /// Silently do nothing if there are no commands.
    // pub allow_empty: bool,
    #[command(subcommand)]
    pub cmd: CommandArgs,
}

#[test]
fn test_cli_args() {
    BufArgs::try_parse_from(&["cmd", "-L", "%", "-c=5", "ls", "%"]).unwrap();
}

pub fn buf_cmd(args: BufArgs) -> ExitStatus {
    let tasks = create_tasks(
        || stdin_lines(EmptyLineHandling::Drop),
        args.cmd,
        args.working_dir,
        args.lines_with.or_else(|| Some("{}".to_owned())),
        args.stdin,
        args.unique,
    );
    let mut task_stack = TaskStack::from(tasks.into_iter().map(TaskType::Pending).collect());
    let to_run = mark_tasks_to_run(
        false,
        args.count.is_none(),
        args.count.unwrap_or(u32::MAX),
        &mut task_stack,
        0,
    );
    if !to_run.is_empty() {
        println!(
            "collected {} commands to run, e.g. {}",
            to_run.len(),
            to_run[0].as_str()
        );
    }
    let statuses = run_tasks(
        to_run,
        args.continue_on_error || args.parallel > 1,
        args.parallel,
        args.quiet,
    );
    let exit_code = statuses
        .iter()
        .map(|entry| entry.value().exit_status())
        .max_by_key(|es| es.code)
        .unwrap_or_else(ExitStatus::ok);
    exit_code
}
