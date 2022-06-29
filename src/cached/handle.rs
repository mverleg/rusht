use ::std::process::exit;

use super::cached;
use super::CacheStatus;
use super::CachedArgs;

pub fn handle_cached(args: CachedArgs) {
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
