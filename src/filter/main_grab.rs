use ::clap::StructOpt;

use ::rusht::filter::handle_grab;
use ::rusht::filter::GrabArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = GrabArgs::from_args();
    handle_grab(args).await
}
