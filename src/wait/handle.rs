use ::std::process::exit;

use super::locked;
use super::LockedArgs;

pub fn handle_locked(args: LockedArgs) {
    match locked(args) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("failed: {}", err);
            exit(1)
        }
    }
}
