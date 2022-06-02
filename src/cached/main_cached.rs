use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht::cached::cached;
use ::rusht::cached::CachedArgs;
use ::rusht::cached::CacheStatus;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    let verbose = ! args.quiet;
    match cached(args) {
        Ok(status) => match status {
            CacheStatus::RanSuccessfully(out) => {
                if verbose {
                    println!("{}", out);
                }
                eprintln!("successfully ran")
            },
            CacheStatus::FromCache(out) => {
                if verbose {
                    println!("{}", out);
                }
                eprintln!("loaded from cache")
            },
            CacheStatus::Failed(exit_code) => {
                eprintln!("the command ran, but it failed and was not cached (code: {})", exit_code);
                exit(exit_code)
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    }
}
