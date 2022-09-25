use crate::ExitStatus;
use ::log::debug;

use crate::rscript::rsh_args::RshArgs;
use crate::rscript::rsh_build::compile_rsh;
use crate::rscript::rsh_context::rsh_context;
use crate::rscript::rsh_read::load_source;
use crate::rscript::rsh_run::execute;

pub fn rsh(args: RshArgs) -> Result<ExitStatus, String> {
    debug!("{:?}", args);
    let context = rsh_context()?;
    let prog = load_source(&args.script)?;
    let state = compile_rsh(&context, prog, &args)?;
    if args.build_only {
        println!(
            "build done, result in {}",
            state
                .exe_path
                .to_str()
                .expect("executable path is not unicode")
        );
        Ok(ExitStatus::ok())
    } else {
        execute(&state, &args)
    }
}
