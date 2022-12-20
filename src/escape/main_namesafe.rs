use ::clap::Parser;

use ::rusht::escape::handle_namesafe;
use ::rusht::escape::NamesafeArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = NamesafeArgs::parse();
    handle_namesafe(args)
}
