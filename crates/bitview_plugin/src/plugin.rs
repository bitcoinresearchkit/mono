use crate::{PluginData, PluginId, PluginStorage};

/// The compatibility contract shared by Bitview's built-in and external plugins.
pub trait Plugin: PluginData + Send + Sync {
    fn storage(&self) -> PluginStorage;

    fn id(&self) -> PluginId {
        self.storage().id()
    }
}
