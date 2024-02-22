use ::clap::Parser;

use ::rusht::rsh::{handle_rsh, RshArgs};
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    // there is now a ready-made solution: #!/usr/bin/env rust-script
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args = RshArgs::parse();
    handle_rsh(args)
}
