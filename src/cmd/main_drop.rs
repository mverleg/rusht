use ::structopt::StructOpt;

use rusht::cmd::{drop_cmd, DropArgs};

fn main() {
    env_logger::init();
    let args = DropArgs::from_args();
    assert!(!args.end, "end not implemented"); //TODO
    drop_cmd(args);
}
