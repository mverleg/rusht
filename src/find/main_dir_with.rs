use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht::find::DirWithArgs;
use ::rusht::find::find_dir_with;
use ::rusht::find::PathModification;

fn main() {
    env_logger::init();
    let args = DirWithArgs::from_args();
    if args.roots.len() > 1 && args.path_modification == PathModification::Relative {
        eprintln!("warning: using multiple roots with relative paths, won't know which file belongs to which root");
    }
    match find_dir_with(args) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line.to_str().unwrap());
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    }
}
