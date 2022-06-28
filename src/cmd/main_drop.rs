use ::clap::StructOpt;

use ::rusht::cmd::DropArgs;
use ::rusht::cmd::handle_drop;

fn main() {
    env_logger::init();
    let args = DropArgs::from_args();
    handle_drop(args)
}
