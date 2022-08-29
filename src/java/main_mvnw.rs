use ::clap::StructOpt;

use ::rusht::java::handle_mvnw;
use ::rusht::java::MvnwArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = MvnwArgs::from_args();
    handle_mvnw(args).await
}
