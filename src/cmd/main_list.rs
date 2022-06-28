use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::cmd::list_cmds;
use ::rusht::cmd::ListArgs;
use ::rusht::cmd::ListErr;
use ::rusht::cmd::handle_list;

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    handle_list(args)
}
