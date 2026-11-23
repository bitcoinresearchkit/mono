//! The serial per-plugin recompute of `bitviewd_bench` (feature `bench-cpu`): the composition's
//! forks run one branch at a time ([`crate::dispatch`]), and every step's process counters (retired
//! instructions, cycles, CPU time), less what the steps nested in it took, are added to its plugin's. Without the feature each item here is
//! inert, so a production build is the code it always was.

#[cfg(feature = "bench-cpu")]
pub use on::*;
#[cfg(not(feature = "bench-cpu"))]
pub use off::*;

use bitview_plugin::{PluginId, UpdateContext};
use brk_error::Result;
use vecdb::Database;

/// Runs now, in a step of `plugin`, the compaction its compute scheduled for a few seconds later
/// (`UpdateContext::compact_database`): serially the next plugin starts at once and would count
/// it. Under the shutdown lock, as the scheduled one. Nothing outside the serial runs.
pub fn compact(plugin: PluginId, db: Option<&Database>, context: UpdateContext<'_>) -> Result<()> {
    let Some(db) = db.filter(|_| enabled()) else {
        return Ok(());
    };
    let _step = step(plugin);
    // Joining wakes the scheduled compaction from its wait, which cancels it and keeps the space
    // marked to reclaim.
    db.sync_bg_tasks()?;
    if db.needs_compaction() {
        let _lock = context.exit().lock();
        db.compact()?;
    }
    Ok(())
}

#[cfg(feature = "bench-cpu")]
mod on {
    use std::{
        collections::BTreeMap,
        iter::Sum,
        mem,
        ops::{Add, AddAssign, Sub},
        sync::{Mutex, OnceLock},
    };

    use bitview_plugin::PluginId;

    /// What a step spends: the process's retired instructions and cycles, and its CPU time in
    /// nanoseconds. Instructions are what co-running work barely moves; the other two are not.
    #[derive(Clone, Copy, Default)]
    pub struct Counts {
        pub instructions: u64,
        pub cycles: u64,
        pub cpu_ns: u64,
    }

    const ZERO: Counts = Counts {
        instructions: 0,
        cycles: 0,
        cpu_ns: 0,
    };

    impl Add for Counts {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            Self {
                instructions: self.instructions + other.instructions,
                cycles: self.cycles + other.cycles,
                cpu_ns: self.cpu_ns + other.cpu_ns,
            }
        }
    }

    impl AddAssign for Counts {
        fn add_assign(&mut self, other: Self) {
            *self = *self + other;
        }
    }

    impl Sub for Counts {
        type Output = Self;
        fn sub(self, other: Self) -> Self {
            Self {
                instructions: self.instructions - other.instructions,
                cycles: self.cycles - other.cycles,
                cpu_ns: self.cpu_ns - other.cpu_ns,
            }
        }
    }

    impl Sum for Counts {
        fn sum<I: Iterator<Item = Self>>(counts: I) -> Self {
            counts.fold(Self::default(), Add::add)
        }
    }

    /// Reads the process's counters; set once, by the benchmark, to start the recompute.
    static CLOCK: OnceLock<fn() -> Counts> = OnceLock::new();
    static LEDGER: Mutex<BTreeMap<&'static str, Counts>> = Mutex::new(BTreeMap::new());
    /// The counters of the steps that ended inside the one now running.
    static NESTED: Mutex<Counts> = Mutex::new(ZERO);

    pub fn start(clock: fn() -> Counts) {
        CLOCK
            .set(clock)
            .expect("the serial CPU recompute started twice");
    }

    pub fn enabled() -> bool {
        CLOCK.get().is_some()
    }

    /// What each plugin spent since the last call, by plugin id.
    pub fn take() -> BTreeMap<&'static str, Counts> {
        mem::take(&mut *LEDGER.lock().expect("serial CPU ledger is poisoned"))
    }

    /// A step of `plugin`: ends when dropped. Steps nest on one thread, as the serial schedule
    /// runs them.
    pub struct Step(Option<Running>);

    struct Running {
        plugin: PluginId,
        clock: fn() -> Counts,
        start: Counts,
        outer_nested: Counts,
    }

    pub fn step(plugin: PluginId) -> Step {
        Step(CLOCK.get().map(|&clock| Running {
            plugin,
            clock,
            start: clock(),
            outer_nested: mem::take(&mut *NESTED.lock().expect("serial nesting is poisoned")),
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
            let mut nested = NESTED.lock().expect("serial nesting is poisoned");
            *LEDGER
                .lock()
                .expect("serial CPU ledger is poisoned")
                .entry(plugin.as_str())
                .or_default() += elapsed - *nested;
            *nested = outer_nested + elapsed;
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
