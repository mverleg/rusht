use ::std::process::exit;

use super::{find_dir_with, DirWithArgs, PathModification};

pub fn handle_dir_with(args: DirWithArgs) {
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
