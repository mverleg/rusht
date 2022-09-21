use crate::rscript::rsh::rsh;
use crate::rscript::rsh_args::RshArgs;
use crate::ExitStatus;

pub fn handle_rsh(args: RshArgs) -> ExitStatus {
    match rsh(args) {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}
