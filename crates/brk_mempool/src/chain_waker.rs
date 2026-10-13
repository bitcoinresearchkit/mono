use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

/// Wakes the driver when the chain publishes, so a pool withheld for that tip goes out at once.
#[derive(Clone, Default)]
pub struct ChainWaker(Arc<(Mutex<bool>, Condvar)>);

impl ChainWaker {
    /// Call after the chain publishes a new tip.
    pub fn published(&self) {
        let (woken, wake) = &*self.0;
        *woken
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        wake.notify_all();
    }

    /// Waits up to `timeout` for [`Self::published`]: whether the chain published since the last
    /// wait.
    pub(crate) fn wait(&self, timeout: Duration) -> bool {
        let (woken, wake) = &*self.0;
        let woken = woken
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (mut woken, _) = wake
            .wait_timeout_while(woken, timeout, |woken| !*woken)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *woken)
    }
}
