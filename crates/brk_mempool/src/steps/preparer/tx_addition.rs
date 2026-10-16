//! A tx unknown to the live store: decode the fetched body, resolve
//! prevouts against the live mempool (same-cycle parents), build a full
//! `Transaction` + `Entry`. Confirmed parents land as `prevout: None` and
//! are filled post-apply by the resolver passed to `Mempool::tick_with`.
//!
//! A tx returning from the graveyard is built the same way: the same txid
//! can come back with a different witness, so the tomb's body is never
//! reused; only its first sighting is kept.

use bitcoin::{Transaction as BitcoinTransaction, TxIn as BitcoinTxIn};
use brk_types::{MempoolEntryInfo, SigOps, Transaction, TxIn, TxOut, TxStatus, Txid, Vout};

use crate::{
    state::TxEntry,
    stores::{TxGraveyard, TxStore},
};

pub struct TxAddition {
    pub tx: Transaction,
    pub entry: TxEntry,
    /// In the raw listing, not only in a (possibly stale) template: only a
    /// listed tx takes over an outpoint another pooled tx already spends.
    pub listed: bool,
}

impl TxAddition {
    /// Resolves prevouts against the live mempool only. Confirmed
    /// parents land with `prevout: None` and are filled by the
    /// resolver supplied to `Mempool::tick_with` in the same cycle.
    ///
    /// Preserves a tomb's original `first_seen`: bitcoind resets the
    /// timestamp on re-acceptance (and GBT synthesis carries "now"), but
    /// the consumer wants the first-ever sighting, not the latest one.
    pub fn new(
        info: &MempoolEntryInfo,
        tx: BitcoinTransaction,
        listed: bool,
        mempool_txs: &TxStore,
        graveyard: &TxGraveyard,
    ) -> Self {
        let total_size = tx.total_size();
        let rbf = tx.input.iter().any(|i| i.sequence.is_rbf());
        let built = Self::build_tx(info, tx, total_size, mempool_txs);
        let mut entry = TxEntry::new(info, total_size as u64, rbf);
        if let Some(tomb) = graveyard.get(&info.txid) {
            entry.first_seen = tomb.record.entry.first_seen;
        }
        Self {
            tx: built,
            entry,
            listed,
        }
    }

    fn build_tx(
        info: &MempoolEntryInfo,
        tx: BitcoinTransaction,
        total_size: usize,
        mempool_txs: &TxStore,
    ) -> Transaction {
        let input = tx
            .input
            .into_iter()
            .map(|txin| Self::build_txin(txin, mempool_txs))
            .collect();
        let mut built = Transaction {
            index: None,
            txid: info.txid,
            version: tx.version.into(),
            total_sigop_cost: SigOps::ZERO,
            weight: info.weight,
            lock_time: tx.lock_time.into(),
            total_size,
            fee: info.fee,
            input,
            output: tx.output.into_iter().map(TxOut::from).collect(),
            status: TxStatus::UNCONFIRMED,
        };
        built.refresh_sigops();
        built
    }

    fn build_txin(txin: BitcoinTxIn, mempool_txs: &TxStore) -> TxIn {
        let prev_txid: Txid = txin.previous_output.txid.into();
        let prev_vout = Vout::from(txin.previous_output.vout);
        let prevout = Self::resolve_prevout(&prev_txid, prev_vout, mempool_txs);

        TxIn {
            // Mempool txs are never coinbase (Core rejects them
            // from the pool entirely).
            is_coinbase: false,
            prevout,
            txid: prev_txid,
            vout: prev_vout,
            script_sig: txin.script_sig,
            witness: txin.witness.into(),
            sequence: txin.sequence.into(),
        }
    }

    fn resolve_prevout(prev_txid: &Txid, prev_vout: Vout, mempool_txs: &TxStore) -> Option<TxOut> {
        let prev = mempool_txs.get(prev_txid)?;
        prev.output
            .get(usize::from(prev_vout))
            .map(|o| TxOut::from((o.script_pubkey.clone(), o.value)))
    }
}
