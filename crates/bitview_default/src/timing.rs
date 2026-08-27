use std::time::{Duration, Instant};

use bitview_plugin::PluginId;
use tracing::info;

#[derive(Clone, Copy)]
pub enum Phase {
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

pub fn timed<T>(phase: Phase, plugin: PluginId, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    let duration_ns = duration_ns(elapsed);

    info!(
        target: "bitview::plugin_timing",
        phase = phase.name(),
        plugin = plugin.as_str(),
        duration_ns,
        "{} {} in {:?}",
        phase.completed(),
        plugin,
        elapsed,
    );

    result
}

fn duration_ns(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_mul(1_000_000_000)
        .saturating_add(u64::from(duration.subsec_nanos()))
}
