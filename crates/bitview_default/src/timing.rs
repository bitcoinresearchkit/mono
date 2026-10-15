use std::{borrow::Cow, time::Instant};

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
    let elapsed = start.elapsed();
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

    Ok(result)
}

/// Imports a plugin, timed under `plugin`, boxed for the composition.
pub fn timed_import<T>(plugin: PluginId, import: impl FnOnce() -> Result<T>) -> Result<Box<T>> {
    timed(Phase::Import, plugin, || import().map(Box::new))
}

/// Computes `plugin`, timed under its own id.
pub fn timed_compute<P: ComputePlugin>(
    plugin: &mut P,
    dependencies: P::Dependencies<'_>,
    context: UpdateContext<'_>,
) -> Result<()> {
    timed(Phase::Compute, plugin.id(), || {
        plugin.compute(dependencies, context)
    })
}

fn plugin_name(plugin: PluginId) -> Cow<'static, str> {
    match plugin.as_str() {
        "op_return" => Cow::Borrowed("OP_RETURN"),
        name if name.contains('_') => Cow::Owned(name.replace('_', " ")),
        name => Cow::Borrowed(name),
    }
}
