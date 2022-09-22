use ::std::env;
use ::std::fs::OpenOptions;
use ::std::io::BufReader;
use ::std::path::PathBuf;
use ::std::time::UNIX_EPOCH;
use std::fs;
use std::time::SystemTime;

use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::log::debug;
use ::sha2::Digest;
use ::sha2::Sha256;

use ::serde::Deserialize;
use ::serde::Serialize;

use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;

pub const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
pub const MAIN_SRC: &str = include_str!("./template/src/main.rs");
pub const DUMMY_ARGS_SRC: &str = include_str!("./template/src/args.rs");
pub const DUMMY_RUN_SRC: &str = include_str!("./template/src/run.rs");

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgState {
    pub name: String,
    pub exe_path: PathBuf,
    pub prog_hash: String,
    pub rsh_hash: u128,
    pub template_hash: String,
    pub last_compile_ts_ms: u128,
}

pub fn derive_prog_state(context: &RshContext, prog: &RshProg) -> ProgState {
    let name = prog.name();
    ProgState {
        name: name.to_owned(),
        exe_path: context.exe_path_for(name),
        prog_hash: calc_hash(vec![&prog.code]),
        rsh_hash: get_rsh_exe_hash(),
        template_hash: calc_hash(vec![CARGO_SRC, MAIN_SRC]),
        last_compile_ts_ms: current_time_ms(),
    }
}

pub fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

pub fn check_should_refresh(current_state: &ProgState, prev_state: &Option<ProgState>) -> bool {
    //TODO @mverleg: make logging conditional?
    let name = &current_state.name;
    if let Some(prev_state) = prev_state {
        if !prev_state.exe_path.is_file() {
            debug!(
                "previous executable for {name} was not found at '{}'",
                prev_state.exe_path.to_string_lossy()
            );
            eprintln!("recompiling {name} because the previous executable has disappeared");
            return true;
        }
        if prev_state.prog_hash != current_state.prog_hash {
            eprintln!("recompiling {name} because the script changed");
            return true;
        }
        if prev_state.rsh_hash != current_state.rsh_hash {
            eprintln!("recompiling {name} because rsh was updated");
            return true;
        }
        if prev_state.template_hash != current_state.template_hash {
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

pub fn write_prog_state(context: &RshContext, state: &ProgState) -> Result<(), String> {
    let pth = context.state_path_for(&state.name);
    let dir_pth = pth
        .parent()
        .expect("could not get parent of state file, but should not be root");
    fs::create_dir_all(dir_pth).map_err(|err| {
        format!(
            "could not create dir '{}', err {}",
            dir_pth.to_string_lossy(),
            err
        )
    })?;
    let state_json = serde_json::to_string(state).map_err(|err| {
        format!(
            "failed to serialize program state for '{}', err {}",
            &state.name, err
        )
    })?;
    debug!(
        "storing {} bytes of program state to '{}'",
        state_json.len(),
        pth.to_string_lossy()
    );
    fs::write(&pth, state_json).map_err(|err| {
        format!(
            "failed to store program state for '{}' into '{}', err {}",
            &state.name,
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
