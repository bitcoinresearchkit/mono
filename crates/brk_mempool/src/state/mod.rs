//! Private state owned exclusively by the update loop.

mod pool;
mod read_only;
pub mod tx_entry;

use brk_types::{MempoolInfo, Txid};
pub(crate) use pool::Pool;
pub use read_only::ReadOnlyState;
pub use tx_entry::TxEntry;

use crate::stores::{AddrTracker, OutpointSpends, TxGraveyard, TxStore};

#[derive(Default)]
pub struct State {
    pub info: MempoolInfo,
    pub txs: TxStore,
    pub addrs: AddrTracker,
    pub outpoint_spends: OutpointSpends,
    pub graveyard: TxGraveyard,
}

impl State {
    pub fn contains_all(&self, live_txids: &[Txid]) -> bool {
        self.txs.len() == live_txids.len() && live_txids.iter().all(|txid| self.txs.contains(txid))
    }
}
