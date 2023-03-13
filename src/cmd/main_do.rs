use ::clap::Parser;

use ::rusht::cmd::DoArgs;
use ::rusht::cmd::handle_do;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = DoArgs::parse();
    handle_do(args)
}
