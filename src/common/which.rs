use ::std::env;
use ::std::time::Instant;

use ::dashmap::DashMap;
use ::lazy_static::lazy_static;
use ::log::info;
use ::log::debug;
use ::log::warn;
use ::which::which_all;

lazy_static! {
    static ref EXE_CACHE: DashMap<String, String> = DashMap::new();
}

pub fn resolve_executable(base_cmd: impl Into<String>) -> String {
    let base_cmd = base_cmd.into();
    let t0 = Instant::now();
    if base_cmd.contains('/') {
        debug!(
            "command {} appears to already be a path, not resolving further",
            base_cmd
        );
        return base_cmd;
    }
    if let Some(cached_exe) = EXE_CACHE.get(&base_cmd) {
        debug!(
            "using cached executable {} for {}",
            cached_exe.value(),
            base_cmd
        );
        return cached_exe.value().clone();
    }
    let do_warn = env::var("RUSHT_SUPPRESS_EXE_RESOLVE").is_err();
    let full_cmd: String = {
        let mut full_cmds = which_all(&base_cmd).unwrap_or_else(|err| {
            panic!(
                "error while trying to find command '{}' on path, err: {}",
                base_cmd, err
            )
        });
        match full_cmds.next() {
            Some(cmd) => {
                if let Some(more_cmd) = full_cmds.next() {
                    if do_warn {
                        info!(
                            "more than one command found for {}: {} and {} (choosing the first)",
                            base_cmd,
                            cmd.to_string_lossy(),
                            more_cmd.to_string_lossy()
                        )
                    }
                }
                cmd.to_str()
                    .unwrap_or_else(|| {
                        panic!(
                            "command {} executable {} not unicode",
                            base_cmd,
                            cmd.to_string_lossy()
                        )
                    })
                    .to_owned()
            }
            None => {
                if do_warn {
                    warn!(
                        "could not find executable for {}, will try to run anyway",
                        base_cmd
                    );
                }
                base_cmd.clone()
            }
        }
    };
    debug!("caching executable {} for {}", &full_cmd, &base_cmd);
    EXE_CACHE.insert(base_cmd, full_cmd.clone());
    let duration = t0.elapsed().as_millis();
    if duration > 200 {
        warn!("resolve_executable slow, took {} ms", duration);
    } else {
        debug!("resolve_executable took {} ms", duration);
    }
    full_cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Panics if not found. Should be safe to assume Cargo exists when running tests.
    #[test]
    fn resolve_cargo() {
        let _exe = resolve_executable("cargo");
    }
}
