use brk_error::Result;

use crate::{Plugin, UpdateContext};

/// Typed computation contract for plugins that participate in the update loop.
pub trait ComputePlugin: Plugin {
    /// Read-only borrowed inputs required for one complete computation.
    type Dependencies<'a>;

    /// Computes this plugin's complete next state. All internal phases and
    /// progress belong to this method; callers only order complete computations.
    /// Returning success must not require an external prepare/push/finish step.
    ///
    /// The runner owns the publication-gate lifecycle so several dependent
    /// plugins can be published together after the complete update succeeds.
    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()>;
}
