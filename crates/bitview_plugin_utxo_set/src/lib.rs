mod compute;
mod dependencies;
mod has;
mod import;

pub use dependencies::Dependencies;
pub use has::HasUtxoSet;

use std::path::PathBuf;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, PerBlock};
use brk_error::Result;
use brk_types::{Bitcoin, Height, Sats, Version};
use statedb::{Creations, History, Reader, Spends, View};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("utxo_set"), Version::ONE);
pub const ID: PluginId = STORAGE.id();

/// Publishes the UTXO set's complete history and its global totals after both producers finish.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    spends_path: PathBuf,
    #[traversable(skip)]
    creations_path: PathBuf,
    #[traversable(skip)]
    history: M::WriteOnly<History>,
    /// Remaining satoshis after each block.
    #[traversable(hidden)]
    pub supply: M::Stored<CachedSeries<Height, Sats>>,
    /// Value of all unspent transaction outputs after the represented block, as Bitcoin Core's
    /// `gettxoutsetinfo` `total_amount` (provably unspendable outputs left out). Reported in BTC;
    /// one BTC equals 100,000,000 satoshis.
    #[traversable(rename = "supply")]
    circulating_supply: LazyPerBlock<Bitcoin, Sats>,
    /// Remaining UTXO count after each block, including zero-value outputs, as Bitcoin Core's
    /// `gettxoutsetinfo` `txouts` (provably unspendable outputs left out).
    pub count: PerBlock<Count, M>,
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
    /// Open the published history; snapshot writes never change what the view reads.
    pub fn view(&self) -> Result<View> {
        Ok(View::open(
            self.db.path(),
            &self.spends_path,
            &self.creations_path,
        )?)
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
