use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::Command;

use ::log::debug;
use ::log::info;
use ::log::trace;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::rscript::rsh_context::RshContext;
use crate::rscript::rsh_program::RshProg;
use crate::rscript::rsh_state::{
    check_should_refresh, read_prog_state, CARGO_SRC, DUMMY_ARGS_SRC, DUMMY_RUN_SRC, MAIN_SRC,
};
use crate::rscript::RshArgs;

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
