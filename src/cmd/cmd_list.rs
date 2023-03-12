use ::std::env::current_dir;

use ::clap::Parser;
use ::log::debug;

use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::stack_pth;

#[derive(Parser, Debug)]
#[command(
    name = "cmlist",
    about = "Show list of pending commands (escaping is not shell-safe), from next to last. See also cmadd, cmdo, cmdrop"
)]
pub struct ListArgs {
    #[arg(short = 'n', long, default_value = "")]
    /// Use the stack from the given namespace instead of the global one
    pub namespace: String,
    #[arg(short = 'p', long, hide_short_help = true)]
    /// Show the path to the stack file, instead of commands
    pub file_path: bool,
    #[arg(short = 'c', long, conflicts_with = "file_path")]
    /// Maximum number of (newest) commands to show.
    pub count: Option<u32>,
    #[arg(short = 'e', long, conflicts_with = "file_path")]
    /// Instead of printing output, use exit code 0 if there are one or more commands pending (1 otherwise).
    pub exit_code: bool,
}

#[test]
fn test_cli_args() {
    ListArgs::try_parse_from(&["cmd", "--file-path"]).unwrap();
    ListArgs::try_parse_from(&["cmd", "-c", "10"]).unwrap();
}

#[derive(Debug, Clone)]
pub enum ListErr {
    Empty,
}

pub fn list_cmds(args: ListArgs) -> Result<Vec<String>, ListErr> {
    debug!("arguments: {:?}", &args);
    if args.file_path {
        let pth = stack_pth(args.namespace);
        return Ok(vec![pth.to_str().unwrap().to_owned()]);
    }
    let tasks = read(args.namespace.clone());
    if tasks.is_empty() {
        if !args.exit_code {
            eprintln!(
                "no commands in namespace '{}'; use the cmadd command",
                args.namespace
            );
        }
        return Err(ListErr::Empty);
    }
    if args.exit_code {
        return Ok(vec![]);
    }
    let tasks_iter = if let Some(count) = args.count {
        tasks.iter().take(count as usize)
    } else {
        tasks.iter().take(usize::MAX)
    };
    let current_dir = current_dir().expect("could not get current working directory");
    Ok(tasks_iter
        .enumerate()
        .map(|(nr, task)| {
            let run_msg = if task.is_running() { "running? " } else { "" };
            let workdir_msg = if current_dir != task.working_dir() {
                format!(" @ {}", task.working_dir().to_string_lossy())
            } else {
                "".to_owned()
            };
            format!(
                "{}  # {}{}{}",
                task.as_cmd_str(),
                run_msg,
                nr + 1,
                workdir_msg
            )
        })
        .collect())
}
