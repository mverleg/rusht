use ::clap::StructOpt;

use ::rusht::cmd::handle_list;
use ::rusht::cmd::ListArgs;
use ::rusht::ExitStatus;

fn main() -> ExitStatus {
    env_logger::init();
    let args = ListArgs::from_args();
    handle_list(args)
}
