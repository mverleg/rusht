use ::structopt::StructOpt;
use ::ustr::Ustr;

use ::rusht::common::{EmptyLineHandling, stdin_lines};
use ::rusht::find::{unique, unique_prefix, UniqueArgs};
use ::rusht::find::DirWithArgs;

fn main() {
    env_logger::init();
    let args = DirWithArgs::from_args();
    dbg!(args);  //TODO @mark:
    unimplemented!();
    // for line in result {
    //     println!("{}", line);
    // }
}
