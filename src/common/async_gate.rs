use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct AsyncGate {
    is_open: Arc<AtomicBool>
}

impl AsyncGate {
    pub fn new() -> Self {
        AsyncGate {
            is_open: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn open(&self) {
        self.is_open.store(true, Ordering::Release);
        //TODO @mverleg: wake others
    }

    pub async fn wait(&self) {
        unimplemented!()
    }
}
