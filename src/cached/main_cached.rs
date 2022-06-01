use std::env::current_dir;
use ::structopt::StructOpt;

use ::rusht::cached::CachedArgs;
use rusht::common::Task;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    // dbg!(&args); //TODO @mark:
    let task = Task::new_split(args.cmd.unpack(), current_dir().unwrap());
    // dbg!(&task);
    eprintln!("cache not ready; always running");  //TODO @mark: TEMPORARY! REMOVE THIS!
    task.execute(false);
}
