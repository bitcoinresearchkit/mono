//! A tx unknown to the live store: build its `Transaction` and `TxEntry`,
//! resolving prevouts from the pool as it stood before this cycle. The others
//! land as `prevout: None`, for `prevouts::fill` after apply.
//!
//! A tx returning from the graveyard is built the same way: the same txid
//! can come back with a different witness, so the tomb's body is never
//! reused; only its first sighting is kept.

use bitcoin::Transaction as BitcoinTransaction;
use brk_types::{MempoolEntryInfo, Transaction, TxOut, Txid, Vout};

use crate::{
    state::{TxEntry, pending_body},
    stores::{TxGraveyard, TxStore},
};

pub struct TxAddition {
    pub tx: Transaction,
    pub entry: TxEntry,
}

impl TxAddition {
    /// Preserves the original `first_seen`, the live record's (a witness variant taking its
    /// place) or the tomb's: bitcoind resets the timestamp on re-acceptance (and GBT synthesis
    /// carries "now"), but the consumer wants the first-ever sighting, not the latest one.
    pub fn new(
        info: &MempoolEntryInfo,
        tx: BitcoinTransaction,
        mempool_txs: &TxStore,
        graveyard: &TxGraveyard,
    ) -> Self {
        let total_size = tx.total_size();
        let rbf = tx.input.iter().any(|i| i.sequence.is_rbf());
        let built = pending_body(
            tx,
            info.txid,
            info.fee,
            info.weight,
            total_size,
            |txid, vout| Self::resolve_prevout(&txid, vout, mempool_txs),
        );
        let mut entry = TxEntry::new(info, total_size as u64, rbf);
        if let Some(record) = mempool_txs.record(&info.txid) {
            entry.first_seen = record.entry.first_seen;
        } else if let Some(tomb) = graveyard.get(&info.txid) {
            entry.first_seen = tomb.record.entry.first_seen;
        }
        Self { tx: built, entry }
    }

    fn resolve_prevout(prev_txid: &Txid, prev_vout: Vout, mempool_txs: &TxStore) -> Option<TxOut> {
        let prev = mempool_txs.get(prev_txid)?;
        prev.output
            .get(usize::from(prev_vout))
            .map(|o| TxOut::from((o.script_pubkey.clone(), o.value)))
    }
}
