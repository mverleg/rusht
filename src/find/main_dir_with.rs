use ::structopt::StructOpt;

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
