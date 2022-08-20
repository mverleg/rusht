use ::clap::StructOpt;

use ::rusht::filter::handle_filter;
use ::rusht::filter::FilterArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = FilterArgs::from_args();
    handle_filter(args).await
}
