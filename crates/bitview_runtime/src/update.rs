use std::time::Instant;

use brk_error::Result;
use tracing::info;
use vecdb::Exit;

use crate::{BootstrapAction, ComputePluginSet};

/// Performs and publishes one complete steady-state update.
pub fn update<P>(plugins: &mut P, exit: &Exit) -> Result<()>
where
    P: ComputePluginSet,
{
    run(plugins, exit, |plugins, exit| {
        plugins.compute(exit)?;
        Ok(())
    })
}

pub fn bootstrap_update<P>(plugins: &mut P, exit: &Exit) -> Result<BootstrapAction>
where
    P: ComputePluginSet,
{
    run(plugins, exit, ComputePluginSet::bootstrap_compute)
}

fn run<P, T>(
    plugins: &mut P,
    exit: &Exit,
    compute: impl FnOnce(&mut P, &Exit) -> Result<T>,
) -> Result<T>
where
    P: ComputePluginSet,
{
    plugins.for_each_plugin(&mut |plugin| plugin.gate().begin_update());

    let start = Instant::now();
    let output = compute(plugins, exit)?;
    plugins.prepare_publish()?;
    plugins.for_each_plugin(&mut |plugin| plugin.gate().finish_update());
    info!("Total compute time: {:?}", start.elapsed());
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use bitview_plugin::{Plugin, PluginGate, PluginId};
    use bitview_traversable::Traversable;

    use super::*;

    #[derive(bitview_traversable::Traversable)]
    struct TestPlugin {
        #[traversable(skip)]
        gate: PluginGate,
    }

    impl Plugin for TestPlugin {
        fn id(&self) -> PluginId {
            PluginId::new("test")
        }

        fn gate(&self) -> &PluginGate {
            &self.gate
        }
    }

    struct TestPlugins {
        plugin: TestPlugin,
        computed_while_closed: AtomicBool,
        prepared_while_closed: AtomicBool,
    }

    impl crate::PluginSet for TestPlugins {
        fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin)) {
            visit(&self.plugin);
        }
    }

    impl ComputePluginSet for TestPlugins {
        fn compute(&mut self, _exit: &Exit) -> Result<()> {
            self.computed_while_closed
                .store(self.plugin.gate().try_read().is_none(), Ordering::Relaxed);
            Ok(())
        }

        fn prepare_publish(&mut self) -> Result<()> {
            self.prepared_while_closed
                .store(self.plugin.gate().try_read().is_none(), Ordering::Relaxed);
            Ok(())
        }
    }

    #[test]
    fn publishes_only_after_compute_and_prepare_complete() -> Result<()> {
        let mut plugins = TestPlugins {
            plugin: TestPlugin {
                gate: PluginGate::new(),
            },
            computed_while_closed: AtomicBool::new(false),
            prepared_while_closed: AtomicBool::new(false),
        };

        update(&mut plugins, &Exit::new())?;

        assert!(plugins.computed_while_closed.load(Ordering::Relaxed));
        assert!(plugins.prepared_while_closed.load(Ordering::Relaxed));
        assert!(plugins.plugin.gate().try_read().is_some());
        Ok(())
    }
}
