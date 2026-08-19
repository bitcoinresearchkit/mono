use bitview_plugin::Plugin;
use brk_error::Result;
use vecdb::Exit;

/// Result of the initial full computation before Bitview starts serving reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapAction {
    /// The composition is ready to publish.
    Ready,
    /// Drop and reimport the complete composition to reclaim transient memory.
    Reimport,
}

/// Statically composed collection of Bitview plugins.
pub trait PluginSet: Send + Sync {
    /// Visits every active plugin exactly once.
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin));
}

/// Writable plugin composition that participates in Bitview's update loop.
pub trait ComputePluginSet: PluginSet {
    /// Performs the complete initial computation before reads are published.
    fn bootstrap_compute(&mut self, exit: &Exit) -> Result<BootstrapAction> {
        self.compute(exit)?;
        Ok(BootstrapAction::Ready)
    }

    /// Performs the composition's typed compute schedule.
    fn compute(&mut self, exit: &Exit) -> Result<()>;

    /// Completes fallible writes before the runtime opens the plugin gates.
    fn prepare_publish(&mut self) -> Result<()> {
        Ok(())
    }
}
