use ::clap::Parser;

use ::rusht::filter::handle_filter;
use ::rusht::filter::FilterArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = BetweenArgs::parse();
    handle_between(args).await
}
