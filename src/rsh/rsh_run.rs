use ::std::collections::HashMap;
use ::std::env;
use ::std::process::Command;

use ::log::debug;

use crate::ExitStatus;
use crate::rsh::rsh_program::RshProg;
use crate::rsh::rsh_state::ProgState;
use crate::rsh::RshArgs;

pub fn execute(prog: &RshProg, state: &ProgState, args: &RshArgs) -> Result<ExitStatus, String> {
    //TODO @mverleg: is this going to be slow like mvn?
    let path = &state.exe_path;
    let pass_args = &args.args;
    debug!(
        "going to execute {} with arguments: [{}]",
        path.to_string_lossy(),
        pass_args.join(", "),
    );
    Command::new(path)
        .args(pass_args)
        .envs(&create_rsh_env(prog, &state))
        .spawn()
        .map_err(|err| {
            format!(
                "failed to execute generated program '{}' which was based on '{}', starting failed, err: {}",
                path.to_string_lossy(),
                args.script.to_string_lossy(),
                err
            )
        })?
        .wait()
        .map(|status| ExitStatus::of_code(status.code()))
        .map_err(|err| {
            format!(
                "failed to execute generated program '{}' which was based on '{}', waiting failed, err: {}",
                path.to_string_lossy(),
                args.script.to_string_lossy(),
                err
            )
        })
}

pub fn create_rsh_env(prog: &RshProg, exe: &&ProgState) -> HashMap<&'static str, String> {
    let mut env: HashMap<&str, String> = HashMap::new();
    let comp_exe_pth = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "".to_owned());
    env.insert("RSH_NAME", exe.name.clone());
    env.insert(
        "RSH_SCRIPT_PATH",
        prog.script_path.to_string_lossy().to_string(),
    );
    env.insert("RSH_LAST_COMPILE_MS", exe.last_compile_ts_ms.to_string());
    env.insert("RSH_COMPILER_PATH", comp_exe_pth);
    env
}
