//! Private state owned exclusively by the update loop.

mod pool;
mod read_only;
pub mod tx_entry;

use brk_types::{MempoolInfo, Transaction, Txid};
pub(crate) use pool::Pool;
pub use read_only::ReadOnlyState;
pub use tx_entry::TxEntry;

use crate::stores::{AddrTracker, OutpointSpends, TxGraveyard, TxStore, tx_store::TxRecord};

#[derive(Default)]
pub struct State {
    pub info: MempoolInfo,
    pub txs: TxStore,
    pub addrs: AddrTracker,
    pub outpoint_spends: OutpointSpends,
    pub graveyard: TxGraveyard,
    /// The last applied poll's template transactions the listing lacks (left Core's pool since
    /// it built the template), in its order: block 0's bodies alone, never pooled.
    pub template_only: Vec<TxRecord>,
}

impl State {
    pub fn contains_all(&self, live_txids: &[Txid]) -> bool {
        self.txs.len() == live_txids.len() && live_txids.iter().all(|txid| self.txs.contains(txid))
    }

    /// Folds into the address view each waiting one of `txids` that's ready (resolved, no pooled
    /// parent waiting), then what of its pooled descendants it held back.
    pub fn fold_ready(&mut self, txids: impl IntoIterator<Item = Txid>) {
        let mut next: Vec<Txid> = txids.into_iter().collect();
        while let Some(txid) = next.pop() {
            if !self.addrs.is_waiting(&txid) {
                continue;
            }
            let Some(record) = self.txs.record(&txid) else {
                continue;
            };
            let tx = &record.tx;
            if !tx.is_resolved() || tx.input.iter().any(|i| self.addrs.is_waiting(&i.txid)) {
                continue;
            }
            self.addrs.fold(record);
            next.extend(self.pooled_children(tx));
        }
    }

    /// Folds back out what descends from `txid`, pooled and waiting now: none folds in ahead of
    /// a parent.
    pub fn hold_descendants(&mut self, txid: &Txid) {
        let Some(record) = self.txs.record(txid) else {
            return;
        };
        let mut next = self.pooled_children(&record.tx);
        while let Some(child) = next.pop() {
            // A waiting one's descendants wait too.
            if self.addrs.is_waiting(&child) {
                continue;
            }
            let Some(record) = self.txs.record(&child) else {
                continue;
            };
            self.addrs.unfold(record);
            next.extend(self.pooled_children(&record.tx));
        }
    }

    /// The pooled transactions spending `tx`'s outputs, each once.
    pub fn pooled_children(&self, tx: &Transaction) -> Vec<Txid> {
        self.outpoint_spends
            .children(&self.txs, tx)
            .into_iter()
            .map(|child| child.entry.txid)
            .collect()
    }
}
