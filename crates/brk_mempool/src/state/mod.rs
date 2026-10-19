//! Private state owned exclusively by the update loop.

mod pool;
mod read_only;
pub mod tx_entry;

use brk_types::{MempoolInfo, Txid};
pub(crate) use pool::Pool;
pub use read_only::ReadOnlyState;
use rustc_hash::FxHashSet;
pub use tx_entry::TxEntry;

use crate::stores::{AddrTracker, OutpointSpends, TxGraveyard, TxStore};

#[derive(Default)]
pub struct State {
    pub info: MempoolInfo,
    pub txs: TxStore,
    pub addrs: AddrTracker,
    pub outpoint_spends: OutpointSpends,
    pub graveyard: TxGraveyard,
    /// The last applied poll's template txids absent from the raw listing.
    /// One still pooled and listed now reclaims its outpoints. Empty at every
    /// pool publication, so a restored state starts it empty.
    pub template_only: FxHashSet<Txid>,
}

impl State {
    pub fn contains_all(&self, live_txids: &[Txid]) -> bool {
        self.txs.len() == live_txids.len() && live_txids.iter().all(|txid| self.txs.contains(txid))
    }
}
