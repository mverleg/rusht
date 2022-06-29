use ::clap::StructOpt;

use ::rusht::cmd::handle_add;
use ::rusht::cmd::AddArgs;

//TODO: option to deduplicate tasks
//TODO: run inside Docker?
//TODO: source bashrc/profile
//TODO: set default command for when stack is empty

fn main() {
    env_logger::init();
    let args = AddArgs::from_args();
    handle_add(args)
}
