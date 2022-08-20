use ::std::process::exit;
use std::fmt;
use std::fmt::Formatter;
use std::process::{ExitCode, Termination};

use ::log::warn;

pub fn fail(msg: impl AsRef<str>) -> ! {
    let msg = msg.as_ref();
    warn!("{}", msg);
    eprintln!("{}", msg);
    debug_assert!(false, "explicit `fail` was called");
    exit(2)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExitStatus {
    pub code: u8,
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl ExitStatus {

    pub fn of(code: u8) -> ExitStatus {
        ExitStatus { code }
    }

    pub fn ok() -> ExitStatus {
        ExitStatus::of(0)
    }

    pub fn err() -> ExitStatus {
        ExitStatus::of(1)
    }
}

impl Termination for ExitStatus {
    fn report(self) -> ExitCode {
        ExitCode::from(self.code)
    }
}
