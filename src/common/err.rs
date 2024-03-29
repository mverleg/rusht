use ::std::fmt;
use ::std::fmt::Formatter;
use ::std::process::exit;
use ::std::process::{ExitCode, Termination};

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

    pub fn of_is_ok(is_ok: bool) -> ExitStatus {
        ExitStatus::of(u8::from(!is_ok))
    }

    pub fn of_code(code: Option<i32>) -> ExitStatus {
        let code = code.map(|val| val.try_into());
        ExitStatus::of(match code {
            Some(Ok(nr)) => nr,
            Some(Err(_)) => u8::MAX,
            None => 1,
        })
    }

    pub fn max(first: ExitStatus, second: ExitStatus) -> ExitStatus {
        if first.code > second.code {
            first
        } else {
            second
        }
    }

    pub fn ok() -> ExitStatus {
        ExitStatus::of(0)
    }

    pub fn err() -> ExitStatus {
        ExitStatus::of(1)
    }

    pub fn code(&self) -> u8 {
        self.code
    }

    pub fn is_ok(&self) -> bool {
        0 == self.code
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

impl Termination for ExitStatus {
    fn report(self) -> ExitCode {
        ExitCode::from(self.code)
    }
}
