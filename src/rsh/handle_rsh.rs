use crate::rsh::rsh::rsh;
use crate::rsh::rsh_args::RshArgs;
use crate::ExitStatus;

pub fn handle_rsh(args: RshArgs) -> ExitStatus {
    match rsh(args) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("{}", err);
            ExitStatus::err()
        }
    }
}
