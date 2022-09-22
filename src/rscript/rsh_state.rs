use ::std::collections::HashMap;
use ::std::env;
use ::std::fs;
use ::std::fs::OpenOptions;
use ::std::io::BufReader;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::time::UNIX_EPOCH;

use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::log::debug;
use ::log::info;
use ::log::trace;
use ::sha2::Digest;
use ::sha2::Sha256;

use ::serde::Deserialize;
use ::serde::Serialize;

use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;
use crate::rscript::RshArgs;

pub const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
pub const MAIN_SRC: &str = include_str!("./template/src/main.rs");
pub const DUMMY_ARGS_SRC: &str = include_str!("./template/src/args.rs");
pub const DUMMY_RUN_SRC: &str = include_str!("./template/src/run.rs");

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgState {
    pub path: PathBuf,
    pub prog_hash: String,
    pub rsh_hash: u128,
    pub template_hash: String,
    pub last_compile_ts_ms: u128,
}

pub fn check_should_refresh(prog: &RshProg, prev_state: &Option<ProgState>) -> bool {
    //TODO @mverleg: make logging conditional?
    let name = prog.name();
    if let Some(prev_state) = prev_state {
        if !prev_state.path.is_file() {
            debug!(
                "previous executable for {name} was not found at '{}'",
                prev_state.path.to_string_lossy()
            );
            eprintln!("recompiling {name} because the previous executable has disappeared");
            return true;
        }
        let prog_hash = calc_hash(vec![&prog.code]);
        if prev_state.prog_hash != prog_hash {
            eprintln!("recompiling {name} because the script changed");
            return true;
        }
        let rsh_hash = get_rsh_exe_hash();
        if prev_state.rsh_hash != rsh_hash {
            eprintln!("recompiling {name} because rsh was updated");
            return true;
        }
        let template_hash = calc_hash(vec![CARGO_SRC, MAIN_SRC]);
        if prev_state.template_hash != template_hash {
            eprintln!("recompiling {name} because rsh has a new template");
            return true;
        }
    } else {
        eprintln!("compiling {name} because no previous state was found");
        return true;
    }
    debug!("using cached value of {name} because nothing changed");
    false
}

pub fn read_prog_state(context: &RshContext, prog: &RshProg) -> Result<Option<ProgState>, String> {
    let pth = context.state_path_for(prog.name());
    if !pth.exists() {
        debug!(
            "no program state for {} at '{}'",
            prog.name(),
            pth.to_string_lossy()
        );
        return Ok(None);
    } else {
        debug!(
            "reading program state for {} from '{}'",
            prog.name(),
            pth.to_string_lossy()
        );
    }
    let reader = OpenOptions::new()
        .read(true)
        .open(&pth)
        .map(BufReader::new)
        .map_err(|err| {
            format!(
                "failed to read rsh state from '{}', err {}",
                pth.to_string_lossy(),
                err
            )
        })?;
    serde_json::from_reader::<_, ProgState>(reader)
        .map(|v| Some(v))
        .map_err(|err| {
            format!(
                "failed to read rsh state from '{}', err {}",
                pth.to_string_lossy(),
                err
            )
        })
}

fn calc_hash(content: Vec<&str>) -> String {
    let mut hasher = Sha256::new();
    for text in content {
        hasher.update(text.as_bytes());
    }
    let hash_out = hasher.finalize();
    encode_config(hash_out, URL_SAFE_NO_PAD)
}

/// Returns the modified time in ms, to be used for checking whether rsh changed.
///
/// Note that this isn't perfect - quite some changes will have no effect, and some behaviour
/// changes may result solely form libraries, which are not included. It's good enough though.
fn get_rsh_exe_hash() -> u128 {
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
            ts_ms
        }
        None => {
            debug!("could not get the timestamp of rsh executable, not including in refresh hash");
            0
        }
    }
}
