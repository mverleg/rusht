use ::clap::Parser;

use ::rusht::cmd::handle_add;
use ::rusht::cmd::AddArgs;
use ::rusht::ExitStatus;

//TODO: option to deduplicate tasks
//TODO: run inside Docker?
//TODO: source bashrc/profile
//TODO: set default command for when stack is empty

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = AddArgs::from_args();
    handle_add(args)
}
