//! Two arrival kinds:
//!
//! - **Fresh** - tx unknown to us. Decode the raw bytes, resolve
//!   prevouts against the live mempool (same-cycle parents), build a
//!   full `Transaction` + `Entry`. Confirmed parents land as
//!   `prevout: None` and are filled post-apply by the resolver passed
//!   to `Mempool::tick_with`.
//! - **Revived** - tx in the graveyard. Rebuild the `Entry` only
//!   (preserving `rbf`, `size`). The Applier exhumes the cached tx
//!   body. No raw decoding.

use bitcoin::{Transaction as BitcoinTransaction, TxIn as BitcoinTxIn};
use brk_types::{MempoolEntryInfo, SigOps, Transaction, TxIn, TxOut, TxStatus, Txid, Vout};

use crate::{
    state::TxEntry,
    stores::{TxStore, TxTombstone},
};

pub enum TxAddition {
    Fresh { tx: Transaction, entry: TxEntry },
    Revived { entry: TxEntry },
}

impl TxAddition {
    /// Resolves prevouts against the live mempool only. Confirmed
    /// parents land with `prevout: None` and are filled by the
    /// resolver supplied to `Mempool::tick_with` in the same cycle.
    pub fn fresh(info: &MempoolEntryInfo, tx: BitcoinTransaction, mempool_txs: &TxStore) -> Self {
        let total_size = tx.total_size();
        let rbf = tx.input.iter().any(|i| i.sequence.is_rbf());
        let built = Self::build_tx(info, tx, total_size, mempool_txs);
        let entry = TxEntry::new(info, total_size as u64, rbf);
        Self::Fresh { tx: built, entry }
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

    /// Preserves the tomb's original `first_seen`: bitcoind resets the
    /// timestamp on re-acceptance (and GBT synthesis carries "now"), but
    /// the consumer wants the first-ever sighting, not the latest one.
    pub fn revived(info: &MempoolEntryInfo, tomb: &TxTombstone) -> Self {
        let mut entry = TxEntry::new(info, tomb.entry.size, tomb.entry.rbf);
        entry.first_seen = tomb.entry.first_seen;
        Self::Revived { entry }
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
            script_sig_asm: (),
            witness: txin.witness.into(),
            sequence: txin.sequence.into(),
            inner_redeem_script_asm: (),
            inner_witness_script_asm: (),
        }
    }

    fn resolve_prevout(prev_txid: &Txid, prev_vout: Vout, mempool_txs: &TxStore) -> Option<TxOut> {
        let prev = mempool_txs.get(prev_txid)?;
        prev.output
            .get(usize::from(prev_vout))
            .map(|o| TxOut::from((o.script_pubkey.clone(), o.value)))
    }
}
