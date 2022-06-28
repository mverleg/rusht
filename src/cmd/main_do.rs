use ::clap::StructOpt;

use ::rusht::cmd::DoArgs;
use ::rusht::cmd::handle_do;

fn main() {
    env_logger::init();
    let args = DoArgs::from_args();
    handle_do(args)
}
