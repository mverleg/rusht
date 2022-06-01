use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht::find::DirWithArgs;
use ::rusht::find::find_dir_with;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    dbg!(&args);  //TODO @mark:
    match find_dir_with(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    }
}
