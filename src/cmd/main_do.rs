use ::clap::StructOpt;

use ::rusht::cmd::handle_do;
use ::rusht::cmd::DoArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init();
    let args = DoArgs::from_args();
    handle_do(args)
}
