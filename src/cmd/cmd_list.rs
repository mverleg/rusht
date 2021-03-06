use ::std::env::current_dir;

use ::clap::StructOpt;
use ::log::debug;

use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::stack_pth;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmlist",
    about = "Show list of pending commands (escaping is not shell-safe), from next to last. See also cmadd, cmdo, cmdrop"
)]
pub struct ListArgs {
    #[structopt(
        short = 'n',
        long,
        default_value = "",
        help = "Use the stack from the given namespace instead of the global one"
    )]
    pub namespace: String,
    #[structopt(
        short = 'p',
        long,
        help = "Show the path to the stack file, instead of commands"
    )]
    pub file_path: bool,
    #[structopt(
        short = 'c',
        long,
        help = "Maximum number of (newest) commands to show",
        conflicts_with = "file-path"
    )]
    pub count: Option<u32>,
    #[structopt(
        short = 'e',
        long,
        help = "Instead of printing output, use exit code 0 if there are one or more commands pending (1 otherwise)",
        conflicts_with = "file-path"
    )]
    pub exit_code: bool,
}

#[test]
fn test_cli_args() {
    use clap::IntoApp;
    ListArgs::into_app().debug_assert()
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
