use ::clap::StructOpt;

use ::rusht::filter::handle_unique;
use ::rusht::filter::FilterArgs;

#[async_std::main]
async fn main() {
    env_logger::init();
    let args = FilterArgs::from_args();
    handle_filter(args).await
}
