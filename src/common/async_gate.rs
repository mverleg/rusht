use ::std::process::ExitStatus as ProcStatus;
use ::std::rc::Rc;

use ::async_std::sync::Mutex;
use ::async_std::sync::MutexGuard;
use ::log::debug;
use ::smallvec::SmallVec;
use ::time::Duration;

use crate::common::{StdWriter, Task};

#[derive(Debug, Clone)]
struct AsyncGate {
    is_open: Rc<Mutex<bool>>,
}

impl AsyncGate {
    pub fn new_closed() -> Self {
        AsyncGate {
            is_open: Rc::new(Mutex::new(false)),
        }
    }

    pub async fn open(&mut self) {
        self.is_open.lock()
    }

    pub async fn wait() {

    }
}
