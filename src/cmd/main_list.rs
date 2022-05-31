use ::std::process::exit;

use ::structopt::StructOpt;
use ::rusht::cmd::{list_cmds, ListArgs};

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    match list_cmds(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
        }
        Err(()) => {
            exit(1);
        }
    }
}
