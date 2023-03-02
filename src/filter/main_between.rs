use ::clap::Parser;

use ::rusht::ExitStatus;
use ::rusht::filter::BetweenArgs;
use ::rusht::filter::FilterArgs;
use ::rusht::filter::handle_between;
use ::rusht::filter::handle_filter;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = BetweenArgs::parse();
    handle_between(args).await
}