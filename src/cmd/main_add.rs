use ::clap::StructOpt;

use ::rusht::cmd::{add_cmd, AddArgs};
use ::rusht::cmd::handle_add;
use ::rusht::common::{EmptyLineHandling, stdin_lines};

//TODO: option to deduplicate tasks
//TODO: run inside Docker?
//TODO: source bashrc/profile
//TODO: set default command for when stack is empty

fn main() {
    env_logger::init();
    let mut args = AddArgs::from_args();
    handle_add(args)
}
