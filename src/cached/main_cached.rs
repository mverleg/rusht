use ::clap::Parser;
use ::env_logger;

use ::rusht::cached::handle_cached;
use ::rusht::cached::CachedArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = CachedArgs::parse();
    handle_cached(args).await
}
