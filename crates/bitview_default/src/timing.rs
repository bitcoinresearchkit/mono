use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use bitview_distribution::walk::{Block, Consumer, Origins};
use bitview_plugin::{ComputePlugin, PluginId, UpdateContext};
use brk_error::Result;
use tracing::info;

#[derive(Clone, Copy)]
enum Phase {
    Import,
    Compute,
}

impl Phase {
    const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Compute => "compute",
        }
    }

    const fn completed(self) -> &'static str {
        match self {
            Self::Import => "Imported",
            Self::Compute => "Computed",
        }
    }
}

/// Runs `f`, logging its duration once it succeeds; a failure propagates unlogged.
fn timed<T>(phase: Phase, plugin: PluginId, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let start = Instant::now();
    let result = f()?;
    log(phase, plugin, start.elapsed());
    Ok(result)
}

fn log(phase: Phase, plugin: PluginId, elapsed: Duration) {
    let duration_ns = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    let plugin_name = plugin_name(plugin);

    info!(
        target: "bitview::plugin_timing",
        {
            "internal.phase" = phase.name(),
            "internal.plugin" = plugin.as_str(),
            "internal.duration_ns" = duration_ns,
        },
        "{} {} in {:.2?}",
        phase.completed(),
        plugin_name,
        elapsed,
    );
}

/// Imports a plugin, timed under `plugin`, boxed for the composition.
pub fn timed_import<T>(plugin: PluginId, import: impl FnOnce() -> Result<T>) -> Result<Box<T>> {
    timed(Phase::Import, plugin, || import().map(Box::new))
}

pub fn timed_compute<P: ComputePlugin>(
    plugin: &mut P,
    dependencies: P::Dependencies<'_>,
    context: UpdateContext<'_>,
) -> Result<()> {
    timed(Phase::Compute, plugin.id(), || {
        plugin.compute(dependencies, context)
    })
}

/// A walk consumer timed as its plugin's compute: making it, its share of the walk, and what it
/// does once the walk completes.
pub struct Timed<C> {
    consumer: C,
    plugin: PluginId,
    /// Spent making it.
    made: Duration,
    /// Spent inside the walk.
    walked: Duration,
}

impl<C: Consumer> Timed<C> {
    pub fn new(plugin: PluginId, make: impl FnOnce() -> Result<C>) -> Result<Self> {
        let start = Instant::now();
        let consumer = make()?;
        Ok(Self {
            consumer,
            plugin,
            made: start.elapsed(),
            walked: Duration::ZERO,
        })
    }

    /// The time it took inside the walk.
    pub fn walked(&self) -> Duration {
        self.walked
    }

    /// Ends the compute with `finish` and logs its whole time.
    pub fn finish(self, finish: impl FnOnce(C)) {
        let start = Instant::now();
        finish(self.consumer);
        log(
            Phase::Compute,
            self.plugin,
            self.made + self.walked + start.elapsed(),
        );
    }

    fn time<T>(&mut self, f: impl FnOnce(&mut C) -> T) -> T {
        let start = Instant::now();
        let result = f(&mut self.consumer);
        self.walked += start.elapsed();
        result
    }
}

impl<C: Consumer> Consumer for Timed<C> {
    fn start(&self) -> usize {
        self.consumer.start()
    }

    fn kept(&self) -> bool {
        self.consumer.kept()
    }

    fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        self.time(|consumer| consumer.restore(origins))
    }

    fn apply(&mut self, block: &Block<'_>) -> Result<()> {
        self.time(|consumer| consumer.apply(block))
    }

    fn save(&mut self) -> Result<()> {
        self.time(|consumer| consumer.save())
    }
}

/// Logs a compute its caller timed.
pub fn log_compute(plugin: PluginId, elapsed: Duration) {
    log(Phase::Compute, plugin, elapsed);
}

fn plugin_name(plugin: PluginId) -> Cow<'static, str> {
    match plugin.as_str() {
        "op_return" => Cow::Borrowed("OP_RETURN"),
        name if name.contains('_') => Cow::Owned(name.replace('_', " ")),
        name => Cow::Borrowed(name),
    }
}
