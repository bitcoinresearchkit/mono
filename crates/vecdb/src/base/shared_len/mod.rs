use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub mod default;

/// Atomic length counter shared across clones.
///
/// Wraps `Arc<AtomicUsize>` to allow multiple vec instances (original and clones)
/// to observe the same length as the original grows.
#[derive(Debug, Clone)]
pub struct SharedLen(Arc<AtomicUsize>);

impl SharedLen {
    /// Creates a new shared length counter.
    pub(crate) fn new(val: usize) -> Self {
        Self(Arc::new(AtomicUsize::new(val)))
    }

    /// Gets the current length.
    #[inline(always)]
    pub(crate) fn get(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    /// Sets the length.
    #[inline]
    pub(crate) fn set(&self, val: usize) {
        self.0.store(val, Ordering::Release);
    }
}
