use ::clap::Parser;

use ::rusht::filter::handle_grab;
use ::rusht::filter::GrabArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = GrabArgs::from_args();
    handle_grab(args).await
}
