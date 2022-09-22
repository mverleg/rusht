use ::std::collections::HashMap;
use ::std::env;
use ::std::fs;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::time::UNIX_EPOCH;

use ::base64::{encode_config, URL_SAFE_NO_PAD};
use ::log::debug;
use ::log::trace;
use ::sha2::Digest;
use ::sha2::Sha256;
use log::info;

use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;

const CARGO_SRC: &str = include_str!("./template/Cargo.toml");
const MAIN_SRC: &str = include_str!("./template/src/main.rs");
const DUMMY_ARGS_SRC: &str = include_str!("./template/src/args.rs");
const DUMMY_RUN_SRC: &str = include_str!("./template/src/run.rs");

pub fn compile_rsh(context: &RshContext, prog: RshProg) -> Result<PathBuf, String> {
    let hash = calc_hash(vec![&get_rsh_exe_hash(), CARGO_SRC, MAIN_SRC, &prog.code]);
    debug!(
        "compiling {} as '{}', hash '{}'",
        prog.name(),
        prog.path.to_string_lossy(),
        hash
    );
    init_clean_template_dir(context);
    todo!();
}

fn init_clean_template_dir(context: &RshContext) -> Result<PathBuf, String> {
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
    write_file(&pth, "Cargo.toml", CARGO_SRC);
    write_file(&pth, "src/main.rs", MAIN_SRC);
    write_file(&pth, "src/run.rs", DUMMY_RUN_SRC);
    write_file(&pth, "src/args.rs", DUMMY_ARGS_SRC);
    info!("going to compile empty template");
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
    Ok(pth)
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
