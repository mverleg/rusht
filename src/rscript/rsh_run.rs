use ::std::path::Path;
use ::std::process::Command;

use ::log::debug;

use crate::rscript::RshArgs;
use crate::ExitStatus;

pub fn execute(exe: &Path, args: &RshArgs) -> Result<ExitStatus, String> {
    //TODO @mverleg: is this going to be slow like mvn?
    debug!(
        "going to execute {} with arguments: [{}]",
        exe.to_string_lossy(),
        args.args.join(", ")
    );
    Command::new(exe)
        .args(&args.args)
        .spawn()
        .map_err(|err| {
            format!(
                "failed to execute generated program '{}' which was based on '{}', starting failed, err: {}",
                exe.to_string_lossy(),
                args.script.to_string_lossy(),
                err
            )
        })?
        .wait()
        .map(|status| ExitStatus::of_code(status.code()))
        .map_err(|err| {
            format!(
                "failed to execute generated program '{}' which was based on '{}', waiting failed, err: {}",
                exe.to_string_lossy(),
                args.script.to_string_lossy(),
                err
            )
        })
}
