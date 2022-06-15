use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::cmd::{do_cmd, DoArgs};

fn main() {
    env_logger::init();
    let args = DoArgs::from_args();
    assert!(!args.parallel > 1, "parallel not implemented"); // TODO

    let all_ok = do_cmd(args);
    exit(if all_ok { 0 } else { 1 })
}
