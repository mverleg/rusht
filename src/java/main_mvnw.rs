use ::clap::StructOpt;

use ::rusht::java::handle_mvnw;
use ::rusht::java::MvnwArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init();
    let args = MvnwArgs::from_args();
    handle_mvnw(args).await
}
