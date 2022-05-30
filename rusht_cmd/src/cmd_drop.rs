use ::structopt::StructOpt;

use crate::cmd_io::read;
use crate::cmd_io::write;
use crate::cmd_type::TaskStack;

#[derive(StructOpt, Debug)]
#[structopt(name = "cmdo", about = "Execute a command and remove it from the stack if successful. See also cmadd, cmlist, cmdrop")]
pub struct DropArgs {
    #[structopt(short = "n", long, default_value = "", help = "Use the stack from the given namespace instead of the global one")]
    pub namespace: String,
    #[structopt(short = "a", long, help = "Drop the entire stack of commands to run", conflicts_with = "count")]
    pub all: bool,
    #[structopt(short = "c", long, default_value = "1", help = "Number of commands to run")]
    pub count: u32,
    #[structopt(short = "e", long, help = "Drop command from the end (last) instead of as the next")]
    pub end: bool,
    #[structopt(short = "q", long, help = "Do not log command(s)")]
    pub quiet: bool,
}

pub fn drop_cmd(args: DropArgs) {
    let mut tasks = read(args.namespace.clone());
    drop_tasks(&mut tasks, args.all, args.count, !args.quiet);
    if !args.quiet {
        if tasks.is_empty() {
            println!("all commands dropped");
        } else {
            println!("{} command(s) left", tasks.len() + 1);
        }
    }
    write(args.namespace, &tasks);
}

fn drop_tasks(tasks: &mut TaskStack, all: bool, drop_count: u32, do_log: bool) {
    let mut drop_cnt = 0;
    while let Some(task) = tasks.pop() {
        if do_log {
            if task.is_running() {
                println!("drop running: {}", task.as_cmd_str());
            } else {
                println!("drop: {}", task.as_cmd_str());
            }
        }
        drop_cnt += 1;
        if !all && drop_cnt == drop_count {
            break
        }
    }
}
