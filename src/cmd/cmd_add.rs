use ::std::env::current_dir;
use ::std::io::stdin;
use ::std::io::Read;
use ::std::path::PathBuf;
use ::std::thread::spawn;

use ::log::debug;
use ::clap::StructOpt;

use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::write;
use crate::common::{fail, CommandArgs, Task};

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
    // #[structopt(short = "f", long, help = "Do not check that the command is known")]
    // pub skip_validation: bool,
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
    #[structopt(
        short = "P",
        long,
        help = "Working directory when running the command. Can use placeholder with -l/-L."
    )]
    pub working_dir: Option<String>,
    #[structopt(subcommand)]
    pub cmd: CommandArgs,
}

pub fn add_cmd(args: AddArgs, line_reader: impl FnOnce() -> Vec<String>) {
    let cmd = args.cmd.unpack();
    let new_tasks = if let Some(templ) = args.lines_with {
        assert!(!templ.is_empty());
        let mut has_placeholder = cmd.iter().any(|part| part.contains(&templ));
        if !has_placeholder
            && (args.working_dir.is_some() && args.working_dir.as_ref().unwrap().contains(&templ))
        {
            has_placeholder = true
        }
        if !has_placeholder {
            fail(format!(
                "did not filter template string '{}' in task or working dir: {}, {:?}",
                templ,
                cmd.join(" "),
                &args.working_dir,
            ))
        }
        debug!("going to read stdin lines");
        line_reader()
            .iter()
            .map(|input| task_from_template(&cmd, input, &templ, &args.working_dir))
            .collect()
    } else {
        spawn(stdin_warning);
        let working_dir = args
            .working_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir().unwrap());
        vec![Task::new_split(cmd, working_dir)]
    };
    debug!("finished constructing {} new tasks", new_tasks.len());
    if new_tasks.is_empty() {
        fail("no tasks found, was stdin empty?");
    }
    let mut stored_tasks = read(args.namespace.clone());
    for task in new_tasks {
        if !args.quiet {
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

fn task_from_template(
    cmd: &[String],
    input: &str,
    templ: &str,
    working_dir: &Option<String>,
) -> Task {
    let parts = cmd.iter().map(|part| part.replace(templ, input)).collect();
    let working_dir = match working_dir {
        Some(dir) => PathBuf::from(dir.replace(templ, input))
            .canonicalize()
            .expect("failed to get absolute path for working directory"),
        None => current_dir().unwrap(),
    };
    Task::new_split(parts, working_dir)
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
