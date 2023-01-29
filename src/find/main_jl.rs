use ::clap::Parser;

use ::rusht::ExitStatus;
use ::rusht::find::DirWithArgs;
use ::rusht::find::handle_dir_with;
use ::rusht::find::handle_jl;
use ::rusht::find::JlArgs;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = JlArgs::parse();
    handle_jl(args).await
}
