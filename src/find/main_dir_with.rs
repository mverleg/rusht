use ::std::process::exit;

use ::clap::StructOpt;

use ::rusht::find::{find_dir_with, DirWithArgs, PathModification};

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
