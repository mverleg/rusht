use crate::ExitStatus;
use ::log::debug;

use crate::rsh::rsh_args::RshArgs;
use crate::rsh::rsh_build::compile_rsh;
use crate::rsh::rsh_context::rsh_context;
use crate::rsh::rsh_read::load_source;
use crate::rsh::rsh_run::execute;

pub fn rsh(args: RshArgs) -> Result<ExitStatus, String> {
    debug!("{:?}", args);
    let context = rsh_context()?;
    let prog = load_source(&args.script)?;
    let state = compile_rsh(&context, &prog, &args)?;
    if args.build_only {
        println!(
            "build done, executable in {}",
            state
                .exe_path
                .to_str()
                .expect("executable path is not unicode")
        );
        Ok(ExitStatus::ok())
    } else {
        execute(&prog, &state, &args)
    }
}
