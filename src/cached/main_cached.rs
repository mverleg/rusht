use ::clap::StructOpt;
use ::env_logger;

use ::rusht::cached::handle_cached;
use ::rusht::cached::CachedArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = CachedArgs::from_args();
    handle_cached(args).await
}
