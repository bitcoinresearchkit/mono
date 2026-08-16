use std::{thread, time::Duration};

use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::Plugin;

/// Keeps a plugin's mutable state stable for one logical read.
pub struct PluginReadGuard {
    pub(crate) guards: Vec<ArcRwLockReadGuard<RawRwLock, ()>>,
}

impl PluginReadGuard {
    /// Acquires multiple plugin gates without retaining a partial set while
    /// an update is running.
    pub fn acquire(plugins: &[&dyn Plugin]) -> Self {
        let mut plugins = plugins.to_vec();
        plugins.sort_unstable_by_key(|plugin| {
            let ptr = *plugin as *const dyn Plugin;
            ptr.cast::<()>() as usize
        });
        plugins.dedup_by(|a, b| std::ptr::addr_eq(*a, *b));

        loop {
            let mut guards = Vec::with_capacity(plugins.len());
            let acquired = plugins.iter().all(|plugin| {
                plugin.gate().try_read().is_some_and(|mut guard| {
                    guards.append(&mut guard.guards);
                    true
                })
            });
            if acquired {
                return Self { guards };
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}
