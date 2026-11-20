//! The serial per-plugin CPU replay of `bitviewd_bench` (feature `bench-cpu`): the composition's
//! forks run one branch at a time ([`crate::dispatch`]), and every step's process CPU, less what
//! the steps nested in it took, is added to its plugin's. Without the feature each item here is
//! inert, so a production build is the code it always was.

#[cfg(feature = "bench-cpu")]
pub use on::*;
#[cfg(not(feature = "bench-cpu"))]
pub use off::*;

#[cfg(feature = "bench-cpu")]
mod on {
    use std::{
        collections::BTreeMap,
        mem,
        sync::{
            Mutex, OnceLock,
            atomic::{AtomicU64, Ordering},
        },
    };

    use bitview_plugin::PluginId;

    /// The process's CPU time in nanoseconds; set once, by the benchmark, to start the replay.
    static CLOCK: OnceLock<fn() -> u64> = OnceLock::new();
    static LEDGER: Mutex<BTreeMap<&'static str, u64>> = Mutex::new(BTreeMap::new());
    /// CPU of the steps that ended inside the one now running.
    static NESTED: AtomicU64 = AtomicU64::new(0);

    pub fn start(clock: fn() -> u64) {
        CLOCK
            .set(clock)
            .expect("the serial CPU replay started twice");
    }

    pub fn enabled() -> bool {
        CLOCK.get().is_some()
    }

    /// The CPU each plugin took since the last call, by plugin id.
    pub fn take() -> BTreeMap<&'static str, u64> {
        mem::take(&mut *LEDGER.lock().expect("serial CPU ledger is poisoned"))
    }

    /// A step of `plugin`: ends when dropped. Steps nest on one thread, as the serial schedule
    /// runs them.
    pub struct Step(Option<Running>);

    struct Running {
        plugin: PluginId,
        clock: fn() -> u64,
        start: u64,
        outer_nested: u64,
    }

    pub fn step(plugin: PluginId) -> Step {
        Step(CLOCK.get().map(|&clock| Running {
            plugin,
            clock,
            start: clock(),
            outer_nested: NESTED.swap(0, Ordering::Relaxed),
        }))
    }

    impl Step {
        pub fn end(self) {}
    }

    impl Drop for Step {
        fn drop(&mut self) {
            let Some(Running { plugin, clock, start, outer_nested }) = self.0.take() else {
                return;
            };
            let elapsed = clock() - start;
            let own = elapsed - NESTED.load(Ordering::Relaxed);
            *LEDGER
                .lock()
                .expect("serial CPU ledger is poisoned")
                .entry(plugin.as_str())
                .or_default() += own;
            NESTED.store(outer_nested + elapsed, Ordering::Relaxed);
        }
    }
}

#[cfg(not(feature = "bench-cpu"))]
mod off {
    use bitview_plugin::PluginId;

    pub const fn enabled() -> bool {
        false
    }

    pub struct Step;

    pub const fn step(_: PluginId) -> Step {
        Step
    }

    impl Step {
        pub const fn end(self) {}
    }
}
