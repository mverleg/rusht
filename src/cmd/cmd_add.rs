use ::clap::Parser;

use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::write;
use crate::cmd::cmd_type::TaskStack;
use crate::cmd::create_cmd::create_tasks;
use crate::common::CommandArgs;

#[derive(Parser, Debug)]
#[command(
    name = "cmadd",
    about = "Add a command to be executed to the stack. See also cmdo, cmlist, cmdrop, cmbuf"
)]
pub struct AddArgs {
    #[arg(short = 'n', long, default_value = "")]
    /// Use the stack from the given namespace instead of the global one.
    pub namespace: String,
    #[arg(short = 'Q', long, hide_short_help = true)]
    /// Do not log the command, but do log the total at the end.
    pub mostly_quiet: bool,
    #[arg(short = 'q', long, conflicts_with = "mostly_quiet")]
    /// Do not log the commands, and also not the total at the end.
    pub quiet: bool,
    #[arg(short = 'e', long)]
    /// Add command at the end (last) instead of as the next.
    pub end: bool,
    #[arg(short = 'l', long)]
    /// Add command for each line of stdin, replacing '{}' by the line.
    pub lines: bool,
    #[arg(short = 'L', long, conflicts_with = "lines")]
    /// Like --lines, but use given replacement placeholder instead of '{}'.
    pub lines_with: Option<String>,
    #[arg(short = 'i', long)]
    /// Stdin to be sent to the command. Can use placeholder with -l/-L.
    pub stdin: Option<String>,
    #[arg(short = 'u', long)]
    /// With --lines or --lines-with, skip any duplicate placeholders.
    pub unique: bool,
    #[arg(short = 'D', long)]
    /// Drop all entries before adding new ones.
    pub replace_existing: bool,
    #[arg(short = '0', long)]
    /// Do not fail if 0 tasks were run due to empty input.
    pub allow_empty: bool,
    #[arg(short = 'P', long)]
    /// Working directory when running the command. Can use placeholder with -l/-L.
    pub working_dir: Option<String>,
    #[command(subcommand)]
    pub cmd: CommandArgs,
    #[arg(long, hide_short_help = true, conflicts_with_all = ["lines_with", "lines"])]
    /// Do not check stdin, no warning for stdin without --lines
    pub ignore_stdin: bool,
}

#[test]
fn test_cli_args() {
    AddArgs::try_parse_from(&["cmd", "-l", "-Q", "-uD", "--", "ls", "{}"]).unwrap();
}

pub fn add_cmd(args: AddArgs, line_reader: impl FnOnce() -> Vec<String>) {
    assert!(
        !args.unique || args.lines_with.is_some(),
        "--unique can only be used with --lines or --lines-with"
    );
    let new_tasks = create_tasks(
        line_reader,
        args.cmd,
        args.working_dir,
        args.lines_with,
        args.stdin,
        args.unique,
        args.ignore_stdin,
    );
    if !args.allow_empty && new_tasks.is_empty() {
        if !args.quiet {
            eprintln!("no tasks found, was stdin empty?");
        }
        return;
    }
    let mut stored_tasks = if args.replace_existing {
        TaskStack::empty()
    } else {
        read(args.namespace.clone())
    };
    for task in new_tasks {
        if !args.quiet && !args.mostly_quiet {
            println!("{}", task.as_str());
        }
        if args.end {
            stored_tasks.add_end(task);
        } else {
            stored_tasks.add(task);
        }
    }
    if !args.quiet {
        println!("{} command(s) pending", stored_tasks.len());
    }
    write(args.namespace, &stored_tasks);
}
