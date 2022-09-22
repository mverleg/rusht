use crate::rscript::rsh_args::RshArgs;
use crate::rscript::rsh_build::compile_rsh;
use crate::rscript::rsh_read::load_source;

pub fn rsh(args: RshArgs) -> Result<(), String> {
    eprintln!("{:?}", args);
    let prog = load_source(&args.script)?;
    compile_rsh(prog)?;
    todo!(); //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
