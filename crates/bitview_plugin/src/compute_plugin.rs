use brk_error::Result;
use vecdb::Database;

use crate::{Plugin, UpdateContext};

/// Typed computation contract for plugins that participate in the update loop.
pub trait ComputePlugin: Plugin {
    /// Read-only borrowed inputs required for one complete computation.
    type Dependencies<'a>;

    /// The database [`Self::compute`] syncs before and compacts after a successful
    /// computation, or `None` when the plugin manages its own storage lifecycle.
    fn database(&self) -> Option<&Database>;

    /// Computes this plugin's complete next state. All internal phases and
    /// progress belong to this method; callers only order complete computations,
    /// then commit them together (where a plugin writes state that changes in
    /// place, the composition's commit writes it with the publication).
    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()>;

    /// Runs one computation inside the storage lifecycle: waits for the previous
    /// update's deferred work, computes, then schedules compaction on success.
    /// The sync runs before `compute_state`, so state it takes survives a failed sync.
    /// Implement [`Self::compute_state`]; do not override this method.
    ///
    /// The composition publishes every plugin's rows together when it commits.
    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        if let Some(db) = self.database() {
            db.sync_bg_tasks()?;
        }
        self.compute_state(dependencies, context)?;
        if let Some(db) = self.database() {
            context.compact_database(db);
        }
        Ok(())
    }
}
