use crate::common::{LineWriter, StdWriter};
use crate::ExitStatus;

use super::cached;
use super::CacheStatus;
use super::CachedArgs;

pub async fn handle_cached(mut args: CachedArgs) -> ExitStatus {
    // sorting is needed for key stability, it is validated later only in debug mode
    args.text.sort();
    args.env.sort();
    let verbose = args.verbose;
    let exit_code = args.exit_code;
    let show_cached_output = !args.no_cached_output;
    let mut writer = StdWriter::stdout();
    match cached(args, &mut writer).await {
        Ok(status) => {
            match status {
                CacheStatus::RanSuccessfully => {
                    if exit_code {
                        if verbose {
                            eprintln!("successfully ran, but --exit-code was provided, so failing exit code")
                        }
                        ExitStatus::err()
                    } else {
                        if verbose {
                            eprintln!("successfully ran")
                        }
                        ExitStatus::ok()
                    }
                }
                CacheStatus::FromCache(out) => {
                    if show_cached_output && !out.is_empty() {
                        writer.write_line(out).await;
                    }
                    if verbose {
                        eprintln!("loaded from cache")
                        //TODO @mverleg: better msg?
                    }
                    ExitStatus::ok()
                }
                CacheStatus::Failed(exit_code) => {
                    eprintln!(
                        "the command ran, but it failed and was not cached (code: {})",
                        exit_code
                    );
                    exit_code
                }
            }
        }
        Err(err) => {
            eprintln!("failed: {}", err);
            ExitStatus::err()
        }
    }
}
