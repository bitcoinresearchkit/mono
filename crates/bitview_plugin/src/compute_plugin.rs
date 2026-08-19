use brk_error::Result;
use vecdb::Exit;

use crate::Plugin;

/// Typed computation contract for plugins that participate in the update loop.
pub trait ComputePlugin: Plugin {
    /// Borrowed inputs required for one computation.
    type Dependencies<'a>;
    /// Value made available to downstream plugins in the same update.
    type Output;

    /// Computes this plugin's next state.
    ///
    /// The runner owns the publication-gate lifecycle so several dependent
    /// plugins can be published together after the complete update succeeds.
    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output>;
}
