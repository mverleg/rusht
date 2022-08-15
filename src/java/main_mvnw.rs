use ::clap::StructOpt;

use ::rusht::java::handle_mvnw;
use ::rusht::java::MvnwArgs;

#[async_std::main]
async fn main() {
    env_logger::init();
    let args = MvnwArgs::from_args();
    handle_mvnw(args).await
}
