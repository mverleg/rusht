use ::clap::StructOpt;

use ::rusht::filter::handle_unique;
use ::rusht::filter::UniqueArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = UniqueArgs::from_args();
    handle_unique(args).await
}
