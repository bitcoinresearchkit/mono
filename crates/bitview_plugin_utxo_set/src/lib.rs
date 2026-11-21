mod compute;
mod dependencies;
mod has;
mod import;
mod urpd;

/// What [`Vecs::urpd`] opens.
pub use bitview_urpd::UrpdHistory;
pub use dependencies::Dependencies;
pub use has::HasUtxoSet;
pub use urpd::UrpdConsumer;

use std::path::PathBuf;

use bitview_distribution::walk::Walk;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, PerBlock};
use brk_error::Result;
use brk_types::{Bitcoin, Sats, Version};
use statedb::{Creations, History, Live, Reader, Spends, View};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("utxo_set"), Version::ONE);
pub const ID: PluginId = STORAGE.id();
/// The URPD history's directory in the plugin's.
const URPD: &str = "urpd";

/// Publishes the UTXO set's complete history and its global totals after both producers finish.
#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    spends_path: PathBuf,
    #[traversable(skip)]
    creations_path: PathBuf,
    /// The history's live state, shared with the read-only clone's views.
    #[traversable(skip)]
    live: Live,
    history: M::WriteOnly<History>,
    urpd: M::WriteOnly<UrpdHistory>,
    /// The one walk of the history, its state kept between updates.
    walk: M::WriteOnly<Walk>,
    /// Remaining satoshis after each block.
    #[traversable(hidden)]
    #[traversable(lead = "Supply: all the bitcoin in unspent outputs.")]
    pub supply: M::Stored<CachedSeries<Sats>>,
    /// Value of all unspent transaction outputs after the represented block, as Bitcoin Core's
    /// `gettxoutsetinfo` `total_amount`: outputs Core leaves out as unspendable (`OP_RETURN`, or a
    /// script over 10,000 bytes) are left out.
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    #[traversable(rename = "supply")]
    #[traversable(lead = "Circulating supply: all the bitcoin in existence, unspent.")]
    circulating_supply: LazyPerBlock<Bitcoin, Sats>,
    /// Remaining UTXO count after each block, including zero-value outputs, as Bitcoin Core's
    /// `gettxoutsetinfo` `txouts`: outputs Core leaves out as unspendable (`OP_RETURN`, or a
    /// script over 10,000 bytes) are left out.
    #[traversable(lead = "UTXO count: how many unspent outputs (separate coins) exist.")]
    pub count: PerBlock<Count, M>,
}

impl Vecs {
    /// The plugin's parts of a walk of its history: the history to read, the walk's state, and
    /// the URPD history the walk feeds (through a [`UrpdConsumer`]) and its readers then read.
    pub fn walk<'a>(
        &'a mut self,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<(Reader<'a>, &'a mut Walk, &'a mut UrpdHistory)> {
        Ok((
            self.history.reader(spends, creations)?,
            &mut self.walk,
            &mut self.urpd,
        ))
    }
}

impl<M: StorageMode> Vecs<M> {
    /// Open the published history; snapshot writes never change what the view reads.
    pub fn view(&self) -> Result<View> {
        Ok(View::open(
            self.db.path(),
            &self.spends_path,
            &self.creations_path,
            self.live.clone(),
        )?)
    }

    /// Open the published URPD history; its appends never change what a reader reads below
    /// its ceiling.
    pub fn urpd(&self) -> Result<UrpdHistory> {
        UrpdHistory::open_reader(&self.db.path().join(URPD))
    }
}
