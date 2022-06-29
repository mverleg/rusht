use ::clap::StructOpt;

use ::rusht::cmd::handle_drop;
use ::rusht::cmd::DropArgs;

fn main() {
    env_logger::init();
    let args = DropArgs::from_args();
    handle_drop(args)
}
