use ::clap::Parser;

use ::rusht::cmd::handle_drop;
use ::rusht::cmd::DropArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = DropArgs::from_args();
    handle_drop(args)
}
