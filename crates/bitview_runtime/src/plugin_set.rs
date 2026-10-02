use bitview_plugin::{Plugin, Publication, UpdateContext};
use brk_error::Result;

/// Result of the initial full computation before Bitview starts serving reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapAction {
    /// The composition is ready to publish.
    Ready,
    /// Drop and reimport the complete composition to reclaim transient memory.
    Reimport,
}

impl BootstrapAction {
    /// Runs the next composition stage only when the current stage is ready.
    pub fn then_compute(self, compute: impl FnOnce() -> Result<()>) -> Result<Self> {
        if self == Self::Ready {
            compute()?;
        }
        Ok(self)
    }
}

/// Statically composed collection of Bitview plugins.
pub trait PluginSet: Send + Sync {
    /// Visits every active plugin exactly once.
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin));
}

/// Writable plugin composition that participates in Bitview's update loop.
pub trait ComputePluginSet: PluginSet {
    /// Shared barrier for the complete pipeline's query-visible mutable state.
    fn publication(&self) -> &Publication;

    /// Performs the complete initial computation before reads are published.
    fn bootstrap_compute(&mut self, context: UpdateContext<'_>) -> Result<BootstrapAction> {
        self.compute(context)?;
        Ok(BootstrapAction::Ready)
    }

    /// Performs the composition's typed compute schedule.
    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()>;

    /// Commits the pipeline-wide publication cursor after every plugin is ready.
    fn commit(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bitview_plugin::{Plugin, PluginId, PluginStorage};
    use bitview_traversable::Traversable;
    use brk_types::Version;

    use super::*;

    #[derive(Traversable)]
    struct TestPlugin {
        #[traversable(skip)]
        storage: PluginStorage,
    }

    impl TestPlugin {
        fn new(id: &'static str) -> Self {
            Self {
                storage: PluginStorage::new(PluginId::new(id), Version::ONE),
            }
        }
    }

    impl Plugin for TestPlugin {
        fn storage(&self) -> PluginStorage {
            self.storage
        }
    }

    #[derive(crate::PluginSet)]
    struct Inner {
        boxed: Box<TestPlugin>,
    }

    #[derive(crate::PluginSet)]
    struct Outer {
        #[plugin_set(flatten)]
        inner: Inner,
        direct: TestPlugin,
        #[plugin_set(skip)]
        _ignored: usize,
    }

    #[test]
    fn derive_visits_direct_boxed_and_flattened_plugins() {
        let plugins = Outer {
            inner: Inner {
                boxed: Box::new(TestPlugin::new("boxed")),
            },
            direct: TestPlugin::new("direct"),
            _ignored: 0,
        };
        let mut ids = Vec::new();

        plugins.for_each_plugin(&mut |plugin| ids.push(plugin.id()));

        assert_eq!(ids, [PluginId::new("boxed"), PluginId::new("direct")]);
    }
}
