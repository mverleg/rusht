use ::std::collections::HashMap;
use ::std::fs;
use ::std::fs::read_to_string;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::Command;

use ::fs_extra::copy_items;
use ::fs_extra::dir::CopyOptions;
use ::fs_extra::remove_items;
use ::log::debug;
use ::log::info;
use ::log::trace;

use crate::rsh::rsh_context::RshContext;
use crate::rsh::rsh_program::RshProg;
use crate::rsh::rsh_run::create_rsh_env;
use crate::rsh::rsh_state::{
    check_should_refresh, derive_prog_state, read_prog_state, write_prog_state, ProgState,
};
use crate::rsh::rsh_state::{CARGO_SRC, DUMMY_ARGS_SRC, DUMMY_RUN_SRC, MAIN_SRC};
use crate::rsh::RshArgs;

pub fn compile_rsh(
    context: &RshContext,
    prog: &RshProg,
    args: &RshArgs,
) -> Result<ProgState, String> {
    let prev_state = read_prog_state(context, prog)?;
    let current_state = derive_prog_state(context, prog);
    if !args.force_rebuild && !check_should_refresh(&current_state, &prev_state) {
        debug!(
            "using cached executable for {} (force_rebuild={})",
            current_state.name, args.force_rebuild
        );
        return Ok(prev_state.unwrap());
    }
    remove_old_exe(prev_state)?;
    let template_pth = init_template_dir(context)?;
    compile_program(
        context,
        prog,
        &current_state,
        template_pth,
        args.keep_generated,
    )?;
    //TODO @mverleg: hash check here

    write_prog_state(context, &current_state)?;
    Ok(current_state)
}

fn remove_old_exe(prev_state: Option<ProgState>) -> Result<(), String> {
    if let Some(state) = prev_state {
        if state.exe_path.is_file() {
            debug!(
                "removing old executable '{}'",
                state.exe_path.to_string_lossy()
            );
            fs::remove_file(&state.exe_path).map_err(|err| {
                format!(
                    "failed to remove old executable '{}', err {}",
                    state.exe_path.to_string_lossy(),
                    err
                )
            })?;
        }
    }
    Ok(())
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
    cargo_compile_dir(&pth, HashMap::new(), false)?;
    Ok(pth)
}

/// Creates, compiles and cleans up the program directory, then returns the path. Returns executable path.
fn compile_program(
    context: &RshContext,
    prog: &RshProg,
    state: &ProgState,
    template_pth: PathBuf,
    keep_generated: bool,
) -> Result<(), String> {
    if keep_generated {
        let build_dir = context.keep_generated_path_for(&prog.name());
        if build_dir.is_dir() {
            debug!(
                "build directory not empty, clearing: '{}'",
                build_dir.to_string_lossy()
            );
            remove_items(&[&build_dir]).map_err(|err| {
                format!(
                    "failed to clear the build directory '{}' before compiling, err {}",
                    build_dir.to_string_lossy(),
                    err
                )
            })?;
        }
        fs::create_dir_all(&build_dir).map_err(|err| {
            format!(
                "failed to create build directory '{}', err {}",
                build_dir.to_string_lossy(),
                err
            )
        })?;
        let res = compile_program_in(&build_dir, prog, state, template_pth);
        println!("generated code is in: {}", build_dir.to_string_lossy());
        res
    } else {
        let build_dir_handle = tempfile::tempdir()
            .map_err(|err| format!("could not create a temporary build directory, err {}", err))?;
        let build_dir = build_dir_handle.path();
        compile_program_in(build_dir, prog, state, template_pth)
    }
}

fn compile_program_in(
    build_dir: &Path,
    prog: &RshProg,
    state: &ProgState,
    template_pth: PathBuf,
) -> Result<(), String> {
    debug!(
        "copying template '{}' to '{}' for program {}",
        template_pth.to_string_lossy(),
        build_dir.to_string_lossy(),
        &state.name,
    );
    let mut opts = CopyOptions::new();
    opts.overwrite = true;
    let template_sub_pths = fs::read_dir(&template_pth)
        .map_err(|err| {
            format!(
                "failed to list entries inside dir '{}', err {}",
                template_pth.to_string_lossy(),
                err
            )
        })?
        .map(|pth| pth.expect("failed to read entry in template dir").path())
        .collect::<Vec<_>>();
    copy_items(&template_sub_pths, build_dir, &opts).map_err(|err| {
        format!(
            "failed to copy directory '{}' to '{}', err {}",
            template_pth.to_string_lossy(),
            build_dir.to_string_lossy(),
            err
        )
    })?;
    debug!(
        "compiling program {} in '{}'",
        &state.name,
        build_dir.to_string_lossy(),
    );
    let cargo_src = CARGO_SRC
        .replace("\"rsh-template\"", &format!("\"{}\"", &state.name))
        .replace(
            "\"Automatically generated\"",
            &format!(
                "\"Automatically generated from {}\"",
                &prog.script_path.to_string_lossy()
            ),
        );
    let run_src = format!("pub fn run(args: Args) {{\n\t{}\n}}", &prog.code);
    write_file(build_dir, "Cargo.toml", &cargo_src)?;
    write_file(build_dir, "src/main.rs", MAIN_SRC)?;
    write_file(build_dir, "src/run.rs", &run_src)?;
    write_file(build_dir, "src/args.rs", DUMMY_ARGS_SRC)?;
    cargo_compile_dir(build_dir, create_rsh_env(prog, &state), true)?;
    let artifact_pth = guess_artifact_path(build_dir, &state.name);
    let exe_path_parent = state
        .exe_path
        .parent()
        .expect("no parent dir, but should not be root");
    debug!(
        "copy {} -> {} (creating {})",
        artifact_pth.to_string_lossy(),
        state.exe_path.to_string_lossy(),
        exe_path_parent.to_string_lossy(),
    );
    fs::create_dir_all(
        state
            .exe_path
            .parent()
            .expect("no parent dir, but should not be root"),
    )
    .map_err(|err| {
        format!(
            "failed to create executable directory '{}', err {}",
            state.exe_path.to_string_lossy(),
            err
        )
    })?;
    assert!(
        artifact_pth.parent().unwrap().is_dir(),
        "no build directory was created (release mode)"
    );
    assert!(
        artifact_pth.is_file(),
        "build directory was created but not executable was produced (release mode)"
    );
    // Use move instead of copy, otherwise Macos finds it suspicious and it gets kill9'ed.
    fs::rename(&artifact_pth, &state.exe_path).or_else(|_| {
        fs::copy(&artifact_pth, &state.exe_path)
            .map(|_| ())
            .map_err(|err| {
                format!(
                    "failed to move or copy artifact '{}' to '{}', err {}",
                    artifact_pth.to_string_lossy(),
                    state.exe_path.to_string_lossy(),
                    err
                )
            })
    })?;
    Ok(())
}

fn guess_artifact_path(build_dir: &Path, name: &str) -> PathBuf {
    let mut artifact_pth = build_dir.to_owned();
    artifact_pth.push("target");
    artifact_pth.push("release");
    artifact_pth.push(name);
    artifact_pth
}

fn cargo_compile_dir(
    pth: &Path,
    mut env: HashMap<&'static str, String>,
    is_offline: bool,
) -> Result<(), String> {
    info!("going to compile Rust code in '{}'", pth.to_string_lossy());
    env.insert("RUSTFLAGS", "-C target-cpu=native".to_owned());
    let mut args = vec!["build", "--release"];
    if is_offline {
        args.push("--offline");
    }
    let exit_code = Command::new("cargo")
        .args(&args)
        .current_dir(pth)
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

/// Write the content to the file, creating directories as needed.
/// Skips writing if content is the same, in order to not trigger rebuilds.
fn write_file(base_pth: &Path, file: impl Into<PathBuf>, content: &str) -> Result<(), String> {
    let mut pth = base_pth.to_owned();
    pth.push(file.into());
    if let Ok(existing_content) = read_to_string(&pth) {
        if content == existing_content {
            trace!(
                "skip writing {} bytes to '{}' because the content has not changed",
                content.len(),
                pth.to_string_lossy()
            );
            return Ok(());
        }
    }
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
