use ::clap::StructOpt;

use ::rusht::cmd::handle_do;
use ::rusht::cmd::DoArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"));
    let args = DoArgs::from_args();
    handle_do(args)
}
