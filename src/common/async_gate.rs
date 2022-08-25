use ::std::future::Future;
use ::std::pin::Pin;
use ::std::sync::Arc;
use ::std::sync::atomic::AtomicBool;
use ::std::sync::atomic::Ordering;
use ::std::task::Context;
use ::std::task::Poll;
use ::std::task::Waker;
use std::sync::Mutex;

use ::smallvec::smallvec;
use ::smallvec::SmallVec;

/// The AsyncGate is initially created as closed. Any number of async functions can
/// awaits its opening. Once it is opened, the operations will wake up and proceed.
/// It can not be closed again. Opening more than once is safe but does nothing.
#[derive(Debug, Clone)]
pub struct AsyncGate {
    content: Arc<AsyncGateContent>,
}

#[derive(Debug)]
pub struct AsyncGateContent {
    is_open: AtomicBool,
    wakers: Mutex<SmallVec<[Waker; 2]>>,
}

impl AsyncGate {
    /// Create a new, closed, gate.
    pub fn new() -> Self {
        AsyncGate {
            content: Arc::new(AsyncGateContent {
                is_open: AtomicBool::new(false),
                wakers: Mutex::new(smallvec![]),
            })
        }
    }

    /// Open the gate, waking up anyone waiting for it.
    pub fn open(&self) {
        let was_open = self.content.is_open.swap(true, Ordering::Release);
        if ! was_open {
            for waker in &self.content.wakers.lock().drain() {
                waker.wake();
            }
        }
    }

    /// Has `open()` been called by anyone?
    pub fn is_open(&self) -> bool {
        self.content.is_open.load(Ordering::Acquire)
    }

    /// Wait for the gate until someone else opens it.
    pub async fn wait(&self) {
        //pub fn wait(&self) -> AsyncGateFuture {
        if self.content.is_open.load(Ordering::Acquire) {
            return
        }
        unimplemented!()
    }
}

// pub struct AsyncGateFuture(AsyncGate);

impl Future for AsyncGate {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let is_open = self.content.is_open.load(Ordering::Acquire);
        if is_open {
            Poll::Ready(())
        } else {
            self.content.wakers.push(cx.waker());
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
