#![doc = include_str!("../README.md")]

mod compute;
mod dependencies;
mod has;
mod live;
mod metrics;
mod vecs;

pub use dependencies::Dependencies;
pub use has::HasDistributionEntry;
pub use vecs::Vecs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::StorageMode;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_entry"), Version::ONE);
impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

#[cfg(test)]
mod tests;
