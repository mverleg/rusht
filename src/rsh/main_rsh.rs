use ::clap::Parser;

use ::rusht::rsh::{handle_rsh, RshArgs};
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = RshArgs::from_args();
    handle_rsh(args)
}
