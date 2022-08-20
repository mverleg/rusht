use ::clap::Parser;

use ::rusht::observe::handle_mon;
use ::rusht::observe::MonArgs;

#[async_std::main]
async fn main() {
    env_logger::init();
    let args = MonArgs::from_args();
    handle_mon(args).await
}
