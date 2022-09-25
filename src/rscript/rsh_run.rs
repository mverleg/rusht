use ::std::collections::HashMap;
use ::std::process::Command;

use ::log::debug;

use crate::rscript::rsh_state::ProgState;
use crate::rscript::RshArgs;
use crate::ExitStatus;

pub fn execute(exe: &ProgState, args: &RshArgs) -> Result<ExitStatus, String> {
    //TODO @mverleg: is this going to be slow like mvn?
    let path = &exe.exe_path;
    debug!(
        "going to execute {} with arguments: [{}]",
        path.to_string_lossy(),
        args.args.join(", ")
    );
    let mut env: HashMap<&str, &str> = HashMap::new();
    let script_path = exe.script_path.to_string_lossy();
    let last_compile_ts = exe.last_compile_ts_ms.to_string();
    env.insert("RSH_NAME", &exe.name);
    env.insert("RSH_SCRIPT_PATH", &script_path);
    env.insert("RSH_LAST_COMPILE_MS", &last_compile_ts);
    Command::new(&path)
        .args(&args.args)
        .envs(&env)
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
