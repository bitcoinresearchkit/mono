use crate::{PluginData, PluginGate, PluginId};
use vecdb::AnyVec;

/// The compatibility contract shared by Bitview's built-in and external plugins.
pub trait Plugin: PluginData + Send + Sync {
    /// Stable identifier for this plugin.
    fn id(&self) -> PluginId;

    /// Publication gate for query-visible mutable state.
    fn gate(&self) -> &PluginGate;

    /// Whether this plugin can replace existing values in the given vector.
    fn mutates_existing(&self, _vec: &dyn AnyVec) -> bool {
        false
    }
}
