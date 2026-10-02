use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// The HTTP future owns this guard; blocking work owns only a cloned signal.
#[derive(Default)]
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub fn signal(&self) -> Arc<AtomicBool> {
        self.0.clone()
    }
}

impl Drop for Cancellation {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
