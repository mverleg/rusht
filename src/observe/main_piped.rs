use ::clap::Parser;

use ::rusht::observe::handle_piped;
use ::rusht::observe::PipedArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = PipedArgs::from_args();
    handle_piped(args).await
}
