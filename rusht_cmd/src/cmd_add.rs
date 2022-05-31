use ::std::io::stdin;
use ::std::io::BufRead;
use ::std::io::Read;
use ::std::thread::spawn;

use ::log::debug;
use ::structopt::StructOpt;

use crate::cmd_io::fail;
use crate::cmd_io::read;
use crate::cmd_io::write;
use crate::cmd_type::PendingTask;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cmadd",
    about = "Add a command to be executed to the stack. See also cmdo, cmlist, cmdrop"
)]
pub struct AddArgs {
    #[structopt(
        short = "n",
        long,
        default_value = "",
        help = "Use the stack from the given namespace instead of the global one"
    )]
    pub namespace: String,
    #[structopt(short = "q", long, help = "Do not log the command")]
    pub quiet: bool,
    #[structopt(
        short = "e",
        long,
        help = "Add command at the end (last) instead of as the next"
    )]
    pub end: bool,
    #[structopt(short = "f", long, help = "Do not check that the command is known")]
    pub skip_validation: bool,
    #[structopt(
        short = "l",
        long,
        help = "Add command for each line of stdin, replacing '{}' by the line"
    )]
    pub lines: bool,
    #[structopt(
        short = "L",
        long,
        help = "Like --lines, but use given replacement placeholder instead of '{}'",
        conflicts_with = "lines"
    )]
    pub lines_with: Option<String>,
    #[structopt(subcommand)]
    pub cmd: AddArgsExtra,
}
//TODO: option to deduplicate tasks
//TODO: run inside Docker?
//TODO: source bashrc/profile
//TODO: set default command for when stack is empty

#[derive(Debug, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum AddArgsExtra {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

pub fn add_cmd(args: AddArgs) {
    let new_tasks = match args.cmd {
        AddArgsExtra::Cmd(cmd) => {
            if let Some(templ) = args.lines_with {
                assert!(!templ.is_empty());
                if !cmd.iter().any(|part| part.contains(&templ)) {
                    fail(format!(
                        "did not rusht_find template string '{}' in task: {}",
                        templ,
                        cmd.join(" ")
                    ))
                }
                debug!("going to read stdin lines");
                stdin()
                    .lock()
                    .lines()
                    .map(|line| line.unwrap())
                    .inspect(|line| debug!("stdin line: {}", line))
                    .filter(|line| !line.trim().is_empty())
                    .map(|input| task_from_template(&cmd, &input, &templ))
                    .collect()
            } else {
                spawn(stdin_warning);
                vec![PendingTask::new_split(cmd)]
            }
        }
    };
    debug!("finished constructing {} new tasks", new_tasks.len());
    if new_tasks.is_empty() {
        fail("no tasks found, was stdin empty?");
    }
    let mut stored_tasks = read(args.namespace.clone());
    for task in new_tasks {
        if !args.quiet {
            println!("{}", task.as_cmd_str());
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

fn task_from_template(cmd: &[String], input: &str, templ: &str) -> PendingTask {
    PendingTask::new_split(cmd.iter().map(|part| part.replace(templ, input)).collect())
}

fn stdin_warning() {
    let mut buffer = [0u8; 1];
    if let Err(err) = stdin().lock().read(&mut buffer) {
        debug!("failed to read stdin, error {}", err)
    }
    if !buffer.is_empty() {
        eprintln!("found data on stdin, but --lines(-with) not given, so it will be ignored")
    }
}
