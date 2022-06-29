use ::clap::StructOpt;

use ::rusht::cmd::handle_list;
use ::rusht::cmd::ListArgs;

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    handle_list(args)
}
