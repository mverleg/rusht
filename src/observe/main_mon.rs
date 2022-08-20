use ::clap::StructOpt;

use ::rusht::filter::handle_grab;
use ::rusht::filter::GrabArgs;

#[async_std::main]
async fn main() {
    env_logger::init();
    let args = MonArgs::from_args();
    handle_mon(args).await
}
