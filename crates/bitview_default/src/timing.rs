use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

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

    result
}

fn plugin_name(plugin: PluginId) -> Cow<'static, str> {
    match plugin.as_str() {
        "op_return" => Cow::Borrowed("OP_RETURN"),
        name if name.contains('_') => Cow::Owned(name.replace('_', " ")),
        name => Cow::Borrowed(name),
    }
}

fn duration_ns(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_mul(1_000_000_000)
        .saturating_add(u64::from(duration.subsec_nanos()))
}

#[cfg(test)]
mod tests {
    use bitview_plugin::PluginId;

    use super::plugin_name;

    #[test]
    fn formats_plugin_names_for_people() {
        assert_eq!(plugin_name(PluginId::new("mining")), "mining");
        assert_eq!(
            plugin_name(PluginId::new("capital_sentiment")),
            "capital sentiment"
        );
        assert_eq!(plugin_name(PluginId::new("op_return")), "OP_RETURN");
    }
}
