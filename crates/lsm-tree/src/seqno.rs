use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering::Relaxed},
};

/// Shared monotonic identifier generator.
#[derive(Clone, Default, Debug)]
pub struct SequenceNumberCounter(Arc<AtomicU64>);

impl SequenceNumberCounter {
    /// Creates a counter whose next value is `next`.
    #[must_use]
    pub fn new(next: u64) -> Self {
        Self(Arc::new(AtomicU64::new(next)))
    }

    /// Allocates the next value.
    ///
    /// # Panics
    ///
    /// Panics if all sequence numbers have been exhausted.
    #[must_use]
    pub fn next(&self) -> u64 {
        let value = self.0.fetch_add(1, Relaxed);
        assert_ne!(value, u64::MAX, "ran out of sequence numbers");
        value
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic = "ran out of sequence numbers"]
    fn rejects_overflow() {
        let _ = super::SequenceNumberCounter::new(u64::MAX).next();
    }
}
