use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::cmd::list_cmds;
use ::rusht::cmd::ListArgs;
use ::rusht::cmd::ListErr;

fn main() {
    env_logger::init();
    let args = ListArgs::from_args();
    match list_cmds(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
        }
        Err(ListErr::Empty) => {
            exit(1);
        }
    }
}
