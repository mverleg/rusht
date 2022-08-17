use ::std::process::exit;

use super::cached;
use super::CacheStatus;
use super::CachedArgs;

pub fn handle_cached(args: CachedArgs) {
    let verbose = args.verbose;
    let show_cached_output = !args.no_cached_output;
    match cached(args) {
        Ok(status) => match status {
            CacheStatus::RanSuccessfully => {
                if verbose {
                    eprintln!("successfully ran")
                    //TODO @mverleg: better msg?
                }
            }
            CacheStatus::FromCache(out) => {
                if show_cached_output {
                    print!("{}", out);
                }
                if verbose {
                    eprintln!("loaded from cache")
                    //TODO @mverleg: better msg?
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
