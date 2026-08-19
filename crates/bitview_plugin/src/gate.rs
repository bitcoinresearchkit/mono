use std::sync::Arc;

use parking_lot::{ArcRwLockWriteGuard, Mutex, RawRwLock, RwLock};

use crate::{PluginReadGuard, read_guard};

/// Shared publication gate for one Bitview plugin.
///
/// Clones refer to the same gate. An update stays closed until
/// [`finish_update`](Self::finish_update) is called explicitly, so an error
/// cannot expose partially updated mutable state.
#[derive(Clone, Default)]
pub struct PluginGate(Arc<Inner>);

#[derive(Default)]
struct Inner {
    gate: Arc<RwLock<()>>,
    writer: Mutex<Option<ArcRwLockWriteGuard<RawRwLock, ()>>>,
}

impl PluginGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Waits for existing readers, then closes the plugin to new readers.
    /// Calling this while the same update is already running is a no-op.
    pub fn begin_update(&self) {
        let mut writer = self.0.writer.lock();
        if writer.is_some() {
            return;
        }

        *writer = Some(self.0.gate.write_arc());
    }

    /// Publishes the completed update and admits new readers.
    ///
    /// # Panics
    ///
    /// Panics when no update is running.
    pub fn finish_update(&self) {
        let writer = self
            .0
            .writer
            .lock()
            .take()
            .expect("plugin update is not running");
        drop(writer);
    }

    /// Attempts to stabilize this plugin for one logical read.
    ///
    /// This never blocks. Async callers can retry cooperatively while an
    /// update is running without tying up an executor thread.
    pub fn try_read(&self) -> Option<PluginReadGuard> {
        self.0.gate.try_read_arc().map(read_guard::single)
    }

    /// Stabilizes this plugin for one logical synchronous read.
    ///
    /// Async callers should use [`try_read`](Self::try_read) and yield between
    /// attempts instead of blocking an executor thread.
    pub fn read(&self) -> PluginReadGuard {
        read_guard::single(self.0.gate.read_arc())
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread, time::Duration};

    use super::*;

    #[test]
    fn update_waits_for_readers_and_stays_closed_until_published() {
        let gate = PluginGate::new();
        let read = gate.try_read().unwrap();
        let writer_gate = gate.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (closed_tx, closed_rx) = mpsc::channel();

        let writer = thread::spawn(move || {
            started_tx.send(()).unwrap();
            writer_gate.begin_update();
            closed_tx.send(()).unwrap();
            writer_gate
        });

        started_rx.recv().unwrap();
        assert!(closed_rx.try_recv().is_err());

        drop(read);
        closed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let gate = writer.join().unwrap();
        assert!(gate.try_read().is_none());

        gate.finish_update();
        assert!(gate.try_read().is_some());
    }

    #[test]
    fn begin_update_is_idempotent() {
        let gate = PluginGate::new();
        gate.begin_update();
        gate.begin_update();
        assert!(gate.try_read().is_none());

        gate.finish_update();
        assert!(gate.try_read().is_some());
    }
}
