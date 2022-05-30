use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht_cmd::do_cmd;
use ::rusht_cmd::DoArgs;

fn main() {
    env_logger::init();
    let args = DoArgs::from_args();
    assert!(!args.parallel, "parallel not implemented");  // TODO

    let all_ok = do_cmd(args);
    exit(if all_ok { 0 } else { 1 })
}
