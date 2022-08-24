use ::clap::StructOpt;

use ::rusht::find::handle_dir_with;
use ::rusht::find::DirWithArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"));
    let args = DirWithArgs::from_args();
    handle_dir_with(args)
}
