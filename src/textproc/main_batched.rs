use ::clap::Parser;

use ::rusht::ExitStatus;
use ::rusht::textproc::BatchedArgs;
use ::rusht::textproc::handle_batched;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = BatchedArgs::parse();
    handle_batched(args).await
}
