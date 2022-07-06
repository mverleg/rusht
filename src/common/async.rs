use ::std::fmt::{Debug, Formatter};

use ::atomicbox::AtomicBox;

/// This is like a spsc bounded queue with capacity 1
pub struct AsyncValue<T> {
    value: AtomicBox<T>,
}

impl <T: Debug> Debug for AsyncOption<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
