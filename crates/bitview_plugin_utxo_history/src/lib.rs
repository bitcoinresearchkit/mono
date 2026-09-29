use std::path::PathBuf;
mod compute;
mod dependencies;
mod has;
mod import;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, PerBlock};
use brk_error::Result;
use brk_types::{Height, Sats, StoredU64, Version};
use statedb::{Creations, History, Reader, Spends, View};
use vecdb::{Database, Rw, StorageMode};

pub use dependencies::Dependencies;
pub use has::HasUtxoHistory;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("utxo_history"), Version::ONE);
pub const ID: PluginId = STORAGE.id();

/// Publishes complete UTXO history and its global totals after both producers finish.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    path: PathBuf,
    #[traversable(skip)]
    history: M::WriteOnly<History>,
    /// Remaining satoshis after each block.
    pub supply: M::Stored<CachedSeries<Height, Sats>>,
    /// Remaining UTXO count after each block, including zero-value outputs.
    pub count: PerBlock<StoredU64, M>,
}

impl Vecs {
    pub fn reader<'a>(
        &'a self,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Reader<'a>> {
        Ok(self.history.reader(spends, creations)?)
    }
}

impl<M: StorageMode> Vecs<M> {
    /// Open published history while the caller holds the pipeline publication guard.
    pub fn view(&self) -> Result<View> {
        Ok(View::open(&self.path)?)
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
