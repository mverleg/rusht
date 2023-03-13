use ::std::collections::HashSet;
use ::std::env::current_dir;
use ::std::io::Read;
use ::std::io::stdin;
use ::std::path::PathBuf;
use ::std::thread::spawn;

use ::clap::Parser;
use ::log::debug;
use log::warn;

use crate::cmd::cmd_io::read;
use crate::cmd::cmd_io::write;
use crate::cmd::cmd_type::TaskStack;
use crate::common::{CommandArgs, fail, Task};

pub fn create_tasks(
    line_reader: impl FnOnce() -> Vec<String>,
    base_cmd: CommandArgs,
    working_dir: Option<String>,
    lines_with: Option<String>,
    stdin: Option<String>,
    unique: bool,
) -> Vec<Task> {
    let cmd = base_cmd.unpack();
    let new_tasks = if let Some(templ) = lines_with {
        assert!(!templ.is_empty());
        let mut has_placeholder = cmd.iter().any(|part| part.contains(&templ));
        if !has_placeholder{
            if let Some(cwd) = &working_dir {
                has_placeholder |= cwd.contains(&templ);
            }
        }
        if !has_placeholder{
            if let Some(sin) = &stdin {
                has_placeholder |= sin.contains(&templ);
            }
        }
        if !has_placeholder {
            fail(format!("did not filter template string '{}' in task, working dir or stdin (cmd={} ; stdin={:?} ; cwd={:?})",
                    templ, cmd.join(" "), &stdin, &working_dir));
        }
        debug!("going to read stdin lines");
        let mut seen: HashSet<&String> = HashSet::new();
        line_reader()
            .iter()
            .filter(|line| !unique || seen.insert(line))
            .map(|input| task_from_template(&cmd, input, &templ, working_dir.as_ref(), stdin.as_ref()))
            .collect()
    } else {
        spawn(stdin_ignored_warning);
        let working_dir = working_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir().unwrap());
        if let Some(sin) = &stdin {
            if sin.contains("{}") {
                warn!("--stdin contains a default placeholder '{{}}' but --lines/--lines-with are not active so it will not be replaced");
            }
        }
        vec![Task::new_split(cmd, working_dir, stdin)]
    };
    debug!("finished constructing {} new tasks", new_tasks.len());
    new_tasks
}

fn task_from_template(
    cmd: &[String],
    input: &str,
    templ: &str,
    working_dir: Option<&String>,
    stdin: Option<&String>,
) -> Task {
    let parts = cmd.iter().map(|part| part.replace(templ, input)).collect();
    let working_dir = match working_dir {
        Some(dir) => PathBuf::from(dir.replace(templ, input))
            .canonicalize()
            .expect("failed to get absolute path for working directory"),
        None => current_dir().unwrap(),
    };
    Task::new_split(parts, working_dir, stdin.map(String::to_owned))
}

fn stdin_ignored_warning() {
    let mut buffer = [0u8; 1];
    if let Err(err) = stdin().lock().read(&mut buffer) {
        debug!("failed to read stdin, error {}", err)
    }
    if !buffer.is_empty() {
        eprintln!("found data on stdin, but --lines(-with) not given, so it will be ignored")
    }
}
