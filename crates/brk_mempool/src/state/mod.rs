//! Private state owned exclusively by the update loop.

mod ancestry;
mod pool;
mod read_only;
pub mod tx_entry;

use brk_types::{MempoolInfo, Transaction, Txid};
pub(crate) use ancestry::{Ancestry, Block};
pub(crate) use pool::{BehindSlot, Pool};
pub use read_only::ReadOnlyState;
pub use tx_entry::TxEntry;

use crate::stores::{AddrTracker, OutpointSpends, TxGraveyard, TxStore, Upkeep, tx_store::TxRecord};

#[derive(Default)]
pub struct State {
    pub info: MempoolInfo,
    pub txs: TxStore,
    pub addrs: AddrTracker,
    pub outpoint_spends: OutpointSpends,
    pub graveyard: TxGraveyard,
    /// The graveyard's writer-only bookkeeping: when each tombstone is due, what vanished while
    /// something can still have displaced it.
    pub upkeep: Upkeep,
    /// The blocks the graveyard was last told, back a few from that tip.
    pub ancestry: Ancestry,
    /// The last applied poll's template transactions the listing lacks (left Core's pool since
    /// it built the template), in its order: block 0's bodies alone, never pooled.
    pub template_only: Vec<TxRecord>,
}

impl State {
    /// The pool a clean shutdown saved, back in place behind `graveyard`: each record as an
    /// arrival (owning its outpoints, folded in), its tombstone (a tree's pending head, saved
    /// vanished) exhumed. The first cycle sets it against Core's listing.
    pub fn restored(graveyard: TxGraveyard, pool: Vec<TxRecord>) -> Self {
        let mut state = Self {
            upkeep: Upkeep::of(&graveyard),
            graveyard,
            ..Self::default()
        };
        for TxRecord { tx, entry, .. } in pool {
            let key = entry.key();
            state.graveyard.exhume(&entry.txid, &mut state.upkeep);
            state.info.add(entry.vsize, entry.fee);
            state.outpoint_spends.insert_spends(&tx, key);
            state.txs.insert(tx, entry);
            let record = state.txs.record_by_key(&key).expect("just inserted");
            state.addrs.add(record);
        }
        state
    }

    pub fn contains_all(&self, live_txids: &[Txid]) -> bool {
        self.txs.len() == live_txids.len() && live_txids.iter().all(|txid| self.txs.contains(txid))
    }

    /// BIP125: a transaction signals replaceability through an unconfirmed ancestor that does
    /// (Core's `IsRBFOptIn`). Each arrival that signals (itself, or through a pooled parent)
    /// passes it down to its pooled descendants, those that came first included. O(the arrivals'
    /// inputs, and the outputs of what signals).
    pub fn inherit_signaling(&mut self, entered: &[Txid]) {
        let mut next: Vec<Txid> = Vec::new();
        let mut inheriting: Vec<Txid> = Vec::new();
        for txid in entered {
            let Some(record) = self.txs.record(txid) else {
                continue;
            };
            if record.entry.rbf {
                next.push(*txid);
            } else if record
                .tx
                .input
                .iter()
                .any(|input| self.txs.entry(&input.txid).is_some_and(|parent| parent.rbf))
            {
                inheriting.push(*txid);
            }
        }
        for txid in inheriting {
            self.txs.set_rbf(&txid, true);
            next.push(txid);
        }
        while let Some(txid) = next.pop() {
            let Some(record) = self.txs.record(&txid) else {
                continue;
            };
            for child in self.pooled_children(&record.tx) {
                if self.txs.set_rbf(&child, true) {
                    next.push(child);
                }
            }
        }
    }

    /// The pooled children of signaling transactions that left (`orphaned`) signal on only by
    /// themselves or through a parent still pooled that does; one that no longer does lets go of
    /// its own descendants' in turn. O(what loses it, and their outputs).
    pub fn recheck_signaling(&mut self, orphaned: Vec<Txid>) {
        let mut next = orphaned;
        while let Some(txid) = next.pop() {
            let Some(record) = self.txs.record(&txid).filter(|record| record.entry.rbf) else {
                continue;
            };
            let tx = &record.tx;
            if tx.input.iter().any(|input| {
                input.sequence < 0xffff_fffe
                    || self.txs.entry(&input.txid).is_some_and(|parent| parent.rbf)
            }) {
                continue;
            }
            let children = self.pooled_children(tx);
            self.txs.set_rbf(&txid, false);
            next.extend(children);
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
