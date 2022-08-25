use ::std::future::Future;
use ::std::pin::Pin;
use ::std::sync::Arc;
use ::std::sync::atomic::AtomicBool;
use ::std::sync::atomic::Ordering;
use ::std::task::Context;
use ::std::task::Poll;

/// The AsyncGate is initially created as closed. Any number of async functions can
/// awaits its opening. Once it is opened, the operations will wake up and proceed.
/// It can not be closed again. Opening more than once is safe but does nothing.
#[derive(Debug, Clone)]
pub struct AsyncGate {
    is_open: Arc<AtomicBool>
}

impl AsyncGate {
    /// Create a new, closed, gate.
    pub fn new() -> Self {
        AsyncGate {
            is_open: Arc::new(AtomicBool::new(false))
        }
    }

    /// Open the gate, waking up anyone waiting for it.
    pub fn open(&self) {
        let was_open = self.is_open.swap(true, Ordering::Release);
        if ! was_open {
            //TODO @mverleg: wake others
            unimplemented!()
        }
    }

    /// Wait for the gate until someone else opens it.
    pub async fn wait(&self) {
        if self.is_open.load(Ordering::Acquire) {
            return
        }
        unimplemented!()
    }
}

impl Future for AsyncGate {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let is_open = self.is_open.load(Ordering::Acquire);
        if is_open {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn already_open() {
        let gate = AsyncGate::new();
        gate.open();
        gate.wait().await;
    }
}
