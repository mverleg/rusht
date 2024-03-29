use ::std::env;
use ::std::fs;
use ::std::fs::OpenOptions;
use ::std::io::BufReader;
use ::std::path::PathBuf;
use ::std::time::SystemTime;
use ::std::time::UNIX_EPOCH;

use ::base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ::log::debug;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::sha2::Digest;
use ::sha2::Sha256;
use base64::Engine;

use crate::rsh::rsh_context::RshContext;
use crate::rsh::rsh_program::RshProg;

pub const CARGO_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resource/rsh/template/Cargo.toml.template"
));
pub const MAIN_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resource/rsh/template/src/main.rs.template"
));
pub const DUMMY_ARGS_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resource/rsh/template/src/args.rs.template"
));
pub const DUMMY_RUN_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resource/rsh/template/src/run.rs.template"
));

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgState {
    pub name: String,
    pub hash_tag: String,
    pub exe_path: PathBuf,
    pub prog_hash: String,
    pub rsh_hash: u128,
    pub template_hash: String,
    pub last_compile_ts_ms: u128,
}

pub fn derive_prog_state(context: &RshContext, prog: &RshProg) -> ProgState {
    let prog_hash = calc_hash(vec![&prog.code]);
    let rsh_hash = get_rsh_exe_hash();
    let template_hash = calc_hash(vec![CARGO_SRC, MAIN_SRC]);
    let hash_tag = calc_hash(vec![
        &prog.name(),
        &prog_hash,
        &rsh_hash.to_string(),
        &template_hash,
    ])[..12]
        .to_owned();
    let exe_path = context.exe_path_for(&format!("{}-{}", prog.name(), &hash_tag));
    ProgState {
        name: prog.name(),
        hash_tag,
        exe_path,
        prog_hash,
        rsh_hash,
        template_hash,
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
    let pth = context.state_path_for(&prog.name());
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
        .map(Some)
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
    URL_SAFE_NO_PAD.encode(hash_out)
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
                "rsh at '{}' was last changed at timestamp(ms) {}",
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
