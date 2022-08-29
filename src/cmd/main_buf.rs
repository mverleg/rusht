use ::clap::StructOpt;

use ::rusht::cmd::BufArgs;
use ::rusht::ExitStatus;
use rusht::cmd::handle_buf;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = BufArgs::from_args();
    handle_buf(args)
}
