use ::clap::Parser;
use ::rusht::find::handle_dir_with;
use ::rusht::find::DirWithArgs;
use ::rusht::ExitStatus;
use ::rusht::find::JlArgs;

fn main() -> ExitStatus {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = JlArgs::parse();
    handle_jl(args)
}
