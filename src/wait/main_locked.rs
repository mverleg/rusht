use ::clap::StructOpt;
use ::env_logger;

use ::rusht::wait::locked;
use ::rusht::wait::LockedArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"));
    let args = LockedArgs::from_args();
    match locked(args) {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("failed: {}", err);
            ExitStatus::err()
        }
    }
}
