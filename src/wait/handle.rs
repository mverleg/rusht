use crate::ExitStatus;

use super::locked;
use super::LockedArgs;

pub fn handle_locked(args: LockedArgs) -> ExitStatus {
    match locked(args) {
        Ok(()) => ExitStatus::ok(),
        Err(err) => {
            eprintln!("failed: {}", err);
            ExitStatus::err()
        }
    }
}
