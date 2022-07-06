use ::clap::StructOpt;

use ::rusht::filter::handle_grab;
use ::rusht::filter::GrabArgs;

#[async_std::main]
async fn main() {
    env_logger::init();
    let args = GrabArgs::from_args();
    handle_grab(args).await
}
