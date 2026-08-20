use crate::{PluginData, PluginGate, PluginId, PluginStorage};

/// The compatibility contract shared by Bitview's built-in and external plugins.
pub trait Plugin: PluginData + Send + Sync {
    /// Stable identity and root storage schema owned by this plugin.
    fn storage(&self) -> PluginStorage;

    /// Stable identifier for this plugin.
    fn id(&self) -> PluginId {
        self.storage().id()
    }

    /// Publication gate for query-visible mutable state.
    fn gate(&self) -> &PluginGate;
}
