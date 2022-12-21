use ::clap::Parser;

use ::rusht::cmd::handle_buf;
use ::rusht::cmd::BufArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = BufArgs::parse();
    handle_buf(args)
}
