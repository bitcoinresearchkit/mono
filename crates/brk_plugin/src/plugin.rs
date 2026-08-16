use crate::PluginGate;
use vecdb::AnyVec;

/// The compatibility contract shared by BRK's built-in and future plugins.
pub trait Plugin: Send + Sync {
    /// Stable identifier for this plugin.
    fn id(&self) -> &'static str;

    /// Publication gate for query-visible mutable state.
    fn gate(&self) -> &PluginGate;

    /// Whether this plugin can replace existing values in the given vector.
    fn mutates_existing(&self, _vec: &dyn AnyVec) -> bool {
        false
    }
}
