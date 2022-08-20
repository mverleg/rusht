use ::clap::StructOpt;

use ::rusht::cmd::handle_drop;
use ::rusht::cmd::DropArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init();
    let args = DropArgs::from_args();
    handle_drop(args)
}
