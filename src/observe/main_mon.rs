use ::clap::Parser;

use ::rusht::observe::handle_mon;
use ::rusht::observe::MonArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = MonArgs::from_args();
    handle_mon(args).await
}
