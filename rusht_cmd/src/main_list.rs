use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht_cmd::list_cmds;
use ::rusht_cmd::ListArgs;

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    let has_item_code = list_cmds(args);
    exit(if has_item_code { 0 } else { 1 })
}
