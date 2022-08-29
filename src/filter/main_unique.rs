use ::clap::StructOpt;

use ::rusht::filter::handle_unique;
use ::rusht::filter::UniqueArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = UniqueArgs::from_args();
    handle_unique(args).await
}
