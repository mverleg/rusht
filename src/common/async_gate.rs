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
            for waker in self.content.wakers.lock().expect("AsyncGate lock poisoned").drain(..) {
                waker.wake();
            }
        }
    }

    /// Has `open()` been called by anyone?
    pub fn is_open(&self) -> bool {
        self.content.is_open.load(Ordering::Acquire)
    }

    /// Wait for the gate until someone else opens it.
    pub fn wait(&self) -> AsyncGateFuture {
        if self.content.is_open.load(Ordering::Acquire) {
            return AsyncGateFuture(&self)
        }
        unimplemented!()
    }
}

pub struct AsyncGateFuture<'a>(&'a AsyncGate);

impl <'a> Future for AsyncGateFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_open() {
            Poll::Ready(())
        } else {
            self.0.content.wakers.lock().expect("AsyncGate lock poisoned").push(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    use async_std::future::timeout;

    use super::*;

    #[async_std::test]
    async fn already_open() {
        let gate = AsyncGate::new();
        assert!(!gate.is_open());
        gate.open();
        gate.wait().await;
        assert!(gate.is_open());
    }

    #[async_std::test]
    async fn not_open() {
        let gate = AsyncGate::new();
        assert!(!gate.is_open());
        let res = timeout(Duration::from_micros(20), gate.wait()).await;
        assert!(!gate.is_open());
        assert!(res.is_err(), "should timeout");
    }

    #[async_std::test]
    async fn open_while_waiting() {
        let gate = AsyncGate::new();
        assert!(!gate.is_open());
        thread::scope(|_| {
            sleep(Duration::from_millis(10000));
            gate.open()
        });
        gate.wait().await;
    }
}
