use ::std::future::Future;
use ::std::pin::Pin;
use ::std::sync::atomic::AtomicU8;
use ::std::sync::atomic::Ordering;
use ::std::sync::Arc;
use ::std::sync::Mutex;
use ::std::task::Context;
use ::std::task::Poll;
use ::std::task::Waker;
use ::std::time::Duration;

use ::async_std::prelude::FutureExt as AltExt;
use ::async_std::task::sleep;
use ::futures::FutureExt;
use ::smallvec::smallvec;
use ::smallvec::SmallVec;

const PENDING: u8 = 0;
const OK: u8 = 1;
const FAIL: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncGateState {
    Pending,
    Ok,
    Fail,
}

/// AsyncGate is a way for async operations to wait for each other.
/// The AsyncGate is initially created as closed. Any number of async functions can
/// awaits its opening. Once it is opened, the operations will wake up and proceed.
/// It can be opened in successful or failed mode, which waiting processes can detect.
/// It can not be closed again. Opening more than once does nothing (keeps original state).
#[derive(Debug, Clone)]
pub struct AsyncGate {
    content: Arc<AsyncGateContent>,
}

#[derive(Debug)]
pub struct AsyncGateContent {
    is_open: AtomicU8,
    wakers: Mutex<SmallVec<[Waker; 2]>>,
}

impl AsyncGate {
    /// Create a new, closed, gate.
    pub fn new() -> Self {
        AsyncGate {
            content: Arc::new(AsyncGateContent {
                is_open: AtomicU8::new(PENDING),
                wakers: Mutex::new(smallvec![]),
            }),
        }
    }

    /// Open the gate, waking up anyone waiting for it.
    pub fn open(&self, is_ok: bool) {
        let new_state = if is_ok { OK } else { FAIL };
        let prev_state = self.content.is_open.compare_exchange(
            PENDING,
            new_state,
            Ordering::Release,
            Ordering::Acquire,
        );
        if prev_state == Ok(PENDING) {
            for waker in self
                .content
                .wakers
                .lock()
                .expect("AsyncGate lock poisoned")
                .drain(..)
            {
                waker.wake();
            }
        }
    }

    /// Has `open()` been called by anyone?
    pub fn is_open(&self) -> bool {
        self.content.is_open.load(Ordering::Acquire) != PENDING
    }

    /// Get the current state without blocking.
    pub fn peek(&self) -> AsyncGateState {
        match self.content.is_open.load(Ordering::Acquire) {
            PENDING => AsyncGateState::Pending,
            OK => AsyncGateState::Ok,
            FAIL => AsyncGateState::Fail,
            _ => unreachable!(),
        }
    }

    /// Wait for the gate until someone else opens it, then return
    /// whether it was successful (true) or failed (false).
    #[allow(dead_code)]
    pub fn wait(&self) -> AsyncGateFuture {
        AsyncGateFuture(self)
    }

    pub async fn wait_timeout(&self, timeout: &Duration) -> Result<bool, ()> {
        AsyncGateFuture(self)
            .map(Ok)
            .race(sleep(*timeout).map(|()| Err(())))
            .await
    }
}

pub struct AsyncGateFuture<'a>(&'a AsyncGate);

impl<'a> Future for AsyncGateFuture<'a> {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = self.0.peek();
        if AsyncGateState::Ok == status {
            Poll::Ready(true)
        } else if AsyncGateState::Fail == status {
            Poll::Ready(false)
        } else {
            self.0
                .content
                .wakers
                .lock()
                .expect("AsyncGate lock poisoned")
                .push(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use ::std::thread;
    use ::std::thread::sleep;

    use ::async_std::future::timeout;
    use ::futures::future::join;

    use super::*;

    #[async_std::test]
    async fn already_open() {
        let gate = AsyncGate::new();
        assert!(!gate.is_open());
        gate.open(true);
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
        let gate_clone = gate.clone();
        thread::scope(|_| {
            sleep(Duration::from_millis(20));
            gate_clone.open(true)
        });
        assert!(gate.is_open());
        gate.wait().await;
    }

    #[async_std::test]
    async fn multiple_waiters() {
        let gate = AsyncGate::new();
        assert!(!gate.is_open());
        let gate_clone = gate.clone();
        let _ = join(
            join(
                join(gate.wait(), gate.wait()),
                join(gate_clone.wait(), gate.wait()),
            ),
            join(
                join(gate.wait(), async { gate.open(true) }),
                join(gate.wait(), gate_clone.wait()),
            ),
        )
        .await;
        assert!(gate.is_open());
    }
}
