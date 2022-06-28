use ::clap::StructOpt;

use ::rusht::cmd::ListArgs;
use ::rusht::cmd::handle_list;

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    handle_list(args)
}
