use crate::ExitStatus;

use super::LockedArgs;

pub fn handle_locked(_args: LockedArgs) -> ExitStatus {
    unimplemented!("does not seem to work?");
    // seq 10 | cmbuf -p8 sh -c 'locked -f=x sleep 1; echo {}'

    // match locked(_args) {
    //     Ok(()) => ExitStatus::ok(),
    //     Err(err) => {
    //         eprintln!("failed: {}", err);
    //         ExitStatus::err()
    //     }
    // }
}
