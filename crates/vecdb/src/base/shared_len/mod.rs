use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub mod default;

/// Atomic length counter shared across clones.
#[derive(Debug, Clone)]
pub struct SharedLen(Arc<AtomicUsize>);

impl SharedLen {
    pub(crate) fn new(val: usize) -> Self {
        Self(Arc::new(AtomicUsize::new(val)))
    }

    #[inline(always)]
    pub(crate) fn get(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn set(&self, val: usize) {
        self.0.store(val, Ordering::Release);
    }
}
