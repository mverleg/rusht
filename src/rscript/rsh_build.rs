use ::std::env;
use ::std::path::PathBuf;
use ::std::time::UNIX_EPOCH;

use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::log::debug;
use ::sha2::Digest;
use ::sha2::Sha256;

use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;

const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
const MAIN_SRC: &str = include_str!("./template/src/main.rs");

pub fn compile_rsh(context: &RshContext, prog: RshProg) -> Result<PathBuf, String> {
    let hash = compute_hash(vec![&get_rsh_exe_hash(), CARGO_SRC, MAIN_SRC, &prog.code]);
    debug!("compiling '{}', hash '{}'", prog.name(), hash);
    todo!();
}

fn compute_hash(content: Vec<&str>) -> String {
    let mut hasher = Sha256::new();
    for text in content {
        hasher.update(text.as_bytes());
    }
    let hash_out = hasher.finalize();
    encode_config(hash_out, URL_SAFE_NO_PAD)
}

/// Returns the modified time in ms as a string, to be used for checking whether rsh changed.
///
/// Note that this isn't perfect - quite some changes will have no effect, and some behaviour
/// changes may result solely form libraries, which are not included. It's good enough though.
fn get_rsh_exe_hash() -> String {
    match env::current_exe()
        .ok()
        .and_then(|pth| pth.metadata().ok())
        .and_then(|meta| meta.modified().ok())
        .and_then(|modi| modi.duration_since(UNIX_EPOCH).ok())
        .map(|dur| dur.as_millis())
    {
        Some(ts_ms) => {
            debug!(
                "rsh at '{}' was last changed {} ms ago",
                env::current_exe().unwrap().to_string_lossy(),
                ts_ms
            );
            ts_ms.to_string()
        }
        None => {
            debug!("could not get the timestamp of rsh executable, not including in refresh hash");
            "".to_owned()
        }
    }
}
