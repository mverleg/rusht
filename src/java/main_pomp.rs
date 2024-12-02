use ::clap::Parser;

use ::rusht::java::handle_pomp;
use ::rusht::java::PompArgs;
use ::rusht::ExitStatus;

#[async_std::main]
async fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = PompArgs::parse();
    handle_pomp(args)
}
