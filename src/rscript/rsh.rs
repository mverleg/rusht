use ::log::debug;

use crate::rscript::rsh_args::RshArgs;
use crate::rscript::rsh_build::compile_rsh;
use crate::rscript::rsh_context::rsh_context;
use crate::rscript::rsh_read::load_source;

pub fn rsh(args: RshArgs) -> Result<(), String> {
    debug!("{:?}", args);
    let context = rsh_context()?;
    let prog = load_source(&args.script)?;
    let exe = compile_rsh(&context, prog, &args)?;
    if args.build_only {
        println!(
            "build done, result in {}",
            exe.to_str().expect("executable path is not unicode")
        );
    } else {
        todo!();
    }
    Ok(())
}
