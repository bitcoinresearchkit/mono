use std::time::{Duration, Instant};

use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::Plugin;

/// Keeps a Bitview plugin's mutable state stable for one logical read.
pub struct PluginReadGuard {
    guards: Vec<ArcRwLockReadGuard<RawRwLock, ()>>,
}

pub fn single(guard: ArcRwLockReadGuard<RawRwLock, ()>) -> PluginReadGuard {
    PluginReadGuard {
        guards: vec![guard],
    }
}

impl PluginReadGuard {
    /// Waits up to `timeout` to acquire multiple plugin gates without
    /// retaining a partial set while an update is running.
    pub fn acquire_for(plugins: &[&dyn Plugin], timeout: Duration) -> Option<Self> {
        let mut plugins = plugins.to_vec();
        plugins.sort_unstable_by_key(|plugin| {
            let ptr = *plugin as *const dyn Plugin;
            ptr.cast::<()>() as usize
        });
        plugins.dedup_by(|a, b| std::ptr::addr_eq(*a, *b));
        let started = Instant::now();

        loop {
            let mut guards = Vec::with_capacity(plugins.len());
            let blocked = plugins.iter().find(|plugin| {
                let Some(mut guard) = plugin.gate().try_read() else {
                    return true;
                };
                guards.append(&mut guard.guards);
                false
            });
            let Some(blocked) = blocked else {
                return Some(Self { guards });
            };

            drop(guards);
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return None;
            }
            drop(blocked.gate().read_for(remaining)?);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread, time::Duration};

    use bitview_traversable::Traversable;
    use brk_types::Version;

    use super::*;
    use crate::{PluginGate, PluginId, PluginStorage};

    #[derive(bitview_traversable::Traversable)]
    struct TestPlugin {
        #[traversable(skip)]
        gate: PluginGate,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                gate: PluginGate::new(),
            }
        }
    }

    impl Plugin for TestPlugin {
        fn storage(&self) -> PluginStorage {
            PluginStorage::new(PluginId::new("test"), Version::ONE)
        }

        fn gate(&self) -> &PluginGate {
            &self.gate
        }
    }

    #[test]
    fn multi_read_releases_partial_set_while_waiting() {
        let first = TestPlugin::new();
        let second = TestPlugin::new();
        second.gate.begin_update();

        thread::scope(|scope| {
            let reader = scope.spawn(|| {
                PluginReadGuard::acquire_for(
                    &[&first as &dyn Plugin, &second as &dyn Plugin],
                    Duration::from_secs(1),
                )
            });

            thread::sleep(Duration::from_millis(10));
            let first_gate = first.gate.clone();
            let (closed_tx, closed_rx) = mpsc::channel();
            let writer = scope.spawn(move || {
                first_gate.begin_update();
                closed_tx.send(()).unwrap();
            });

            closed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            writer.join().unwrap();
            second.gate.finish_update();
            first.gate.finish_update();

            assert!(reader.join().unwrap().is_some());
        });
    }

    #[test]
    fn multi_read_stops_waiting_at_its_deadline() {
        let first = TestPlugin::new();
        let second = TestPlugin::new();
        second.gate.begin_update();

        assert!(
            PluginReadGuard::acquire_for(
                &[&first as &dyn Plugin, &second as &dyn Plugin],
                Duration::from_millis(10),
            )
            .is_none()
        );

        second.gate.finish_update();
    }
}
