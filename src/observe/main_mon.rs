use ::clap::Parser;

use ::rusht::observe::handle_mon;
use ::rusht::observe::MonArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = MonArgs::parse();
    handle_mon(args).await
}
