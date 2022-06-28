use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::cmd::{do_cmd, DoArgs};
use ::rusht::cmd::handle_do;

fn main() {
    env_logger::init();
    let args = DoArgs::from_args();
    handle_do(args)
}
