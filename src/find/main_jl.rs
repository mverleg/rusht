use ::clap::Parser;

use ::rusht::find::handle_jl;
use ::rusht::find::JlArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = JlArgs::parse();
    handle_jl(args).await
}
