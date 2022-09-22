use crate::rscript::rsh_args::RshArgs;
use crate::rscript::rsh_io::load_source;

pub fn rsh(args: RshArgs) -> Result<(), String> {
    eprintln!("{:?}", args);
    let src = load_source(&args.script)?;
    todo!(); //TODO @mverleg: TEMPORARY! REMOVE THIS!
}
