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

const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
const MAIN_SRC: &str = include_str!("./template/src/main.rs");
const DUMMY_ARGS_SRC: &str = include_str!("./template/src/args.rs");
const DUMMY_RUN_SRC: &str = include_str!("./template/src/run.rs");

pub fn compile_rsh(context: &RshContext, prog: RshProg, args: &RshArgs) -> Result<PathBuf, String> {
    let prev_state = read_prog_state(context, &prog)?;
    if !args.force_rebuild && !check_should_refresh(&prog, &prev_state) {
        return Ok(prev_state.unwrap().path);
    }
    let template_pth = init_template_dir(context)?;
    //TODO @mverleg: hash check here

    todo!();
}

#[derive(Debug, Serialize, Deserialize)]
struct ProgState {
    path: PathBuf,
    prog_hash: String,
    rsh_hash: String,
    template_hash: String,
    last_compile_ts_ms: u128,
}

fn check_should_refresh(prog: &RshProg, prev_state: &Option<ProgState>) -> bool {
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

fn read_prog_state(context: &RshContext, prog: &RshProg) -> Result<Option<ProgState>, String> {
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

/// Creates and compiles a fixed project directory, to cache dependencies. Returns directory.
fn init_template_dir(context: &RshContext) -> Result<PathBuf, String> {
    let pth = context.empty_template_dir();
    debug!(
        "creating clean template in '{}', exists={}",
        pth.to_string_lossy(),
        pth.is_dir()
    );
    fs::create_dir_all(&pth).map_err(|err| {
        format!(
            "could not create dir '{}', err {}",
            pth.to_string_lossy(),
            err
        )
    })?;
    write_file(&pth, "Cargo.toml", CARGO_SRC)?;
    write_file(&pth, "src/main.rs", MAIN_SRC)?;
    write_file(&pth, "src/run.rs", DUMMY_RUN_SRC)?;
    write_file(&pth, "src/args.rs", DUMMY_ARGS_SRC)?;
    cargo_compile_dir(&pth)?;
    Ok(pth)
}

/// Creates, compiles and cleans up the program directory, then returns the path. Returns executable path.
fn compile_program(context: &RshContext) -> Result<ProgState, String> {
    todo!();
    // let pth = context.empty_template_dir();
    // debug!(
    //     "creating clean template in '{}', exists={}",
    //     pth.to_string_lossy(),
    //     pth.is_dir()
    // );
    // fs::create_dir_all(&pth).map_err(|err| {
    //     format!(
    //         "could not create dir '{}', err {}",
    //         pth.to_string_lossy(),
    //         err
    //     )
    // })?;
    // write_file(&pth, "Cargo.toml", CARGO_SRC)?;
    // write_file(&pth, "src/main.rs", MAIN_SRC)?;
    // write_file(&pth, "src/run.rs", DUMMY_RUN_SRC)?;
    // write_file(&pth, "src/args.rs", DUMMY_ARGS_SRC)?;
    // cargo_compile_dir(&pth)?;
    // Ok(pth)
}

fn cargo_compile_dir(pth: &PathBuf) -> Result<(), String> {
    info!("going to compile Rust code in '{}'", pth.to_string_lossy());
    let mut env = HashMap::new();
    env.insert("RUSTFLAGS", "-C target-cpu=native");
    let exit_code = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&pth)
        .envs(&env)
        .spawn()
        .map_err(|err| {
            format!(
                "failed to compile empty cargo template directory '{}', starting failed, err: {}",
                pth.to_string_lossy(),
                err
            )
        })?
        .wait()
        .map_err(|err| {
            format!(
                "failed to compile empty cargo template directory '{}', waiting failed, err: {}",
                pth.to_string_lossy(),
                err
            )
        })?;
    if !exit_code.success() {
        return Err(format!(
            "failed to compile generated code in '{}'",
            pth.to_string_lossy()
        ));
    }
    Ok(())
}

fn write_file(base_pth: &Path, file: impl Into<PathBuf>, content: &str) -> Result<(), String> {
    let mut pth = base_pth.to_owned();
    pth.push(file.into());
    let parent = pth
        .parent()
        .expect("could not get parent, but no root dir expected");
    fs::create_dir_all(parent).map_err(|err| {
        format!(
            "could not create dir '{}' for file '{}', err {}",
            parent.to_string_lossy(),
            pth.to_string_lossy(),
            err
        )
    })?;
    trace!(
        "writing {} bytes to '{}'",
        content.len(),
        pth.to_string_lossy()
    );
    fs::write(&pth, content).map_err(|err| {
        format!(
            "failed to write '{}' for empty template, err {}",
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
