use ::std::process::exit;

use ::structopt::StructOpt;

use ::rusht::cached::cached;
use ::rusht::cached::CacheStatus;
use ::rusht::cached::CachedArgs;

fn main() {
    env_logger::init();
    let args = CachedArgs::from_args();
    let verbose = args.verbose;
    match cached(args) {
        Ok(status) => match status {
            CacheStatus::RanSuccessfully => {
                if verbose {
                    eprintln!("successfully ran")
                }
            }
            CacheStatus::FromCache(out) => {
                print!("{}", out);
                if verbose {
                    eprintln!("loaded from cache")
                }
            }
            CacheStatus::Failed(exit_code) => {
                eprintln!(
                    "the command ran, but it failed and was not cached (code: {})",
                    exit_code
                );
                exit(exit_code)
            }
        },
        Err(err) => {
            eprintln!("failed: {}", err);
            exit(1)
        }
    }
}
