use brk_types::{Transaction, Txid, Vin, Vout};
use rapidhash::fast::RandomState;
use smallvec::SmallVec;

use crate::stores::{ReadOnlyTxStore, cow::CowMap, tx_store::TxRecord};

/// Mempool index from spent outpoint to the txid of the spending mempool tx, looked up against
/// `TxStore` to recover the spender's record.
#[derive(Clone, Default)]
pub struct OutpointSpends {
    map: CowMap<(Txid, Vout), Txid, RandomState>,
    /// Bumped on every mapping change, so a change of owner alone (no
    /// membership change) still republishes.
    revision: u64,
}

impl OutpointSpends {
    /// `spender` owns the outpoints `tx` spends.
    pub fn insert_spends(&mut self, tx: &Transaction, spender: Txid) {
        let mut changed = false;
        for outpoint in Self::spent_outpoints(tx) {
            changed |= self.map.get(&outpoint) != Some(&spender) && {
                self.map.insert(outpoint, spender);
                true
            };
        }
        self.bump_if(changed);
    }

    /// Only removes entries whose stored spender still matches `spender`,
    /// so an outpoint already re-claimed by a later spender is left alone.
    pub fn remove_spends(&mut self, tx: &Transaction, spender: Txid) {
        let mut changed = false;
        for outpoint in Self::spent_outpoints(tx) {
            if self.map.get(&outpoint) == Some(&spender) {
                self.map.remove(&outpoint);
                changed = true;
            }
        }
        self.bump_if(changed);
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn bump_if(&mut self, changed: bool) {
        if changed {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// The txid of the live spender of `outpoint`.
    #[inline]
    pub fn get(&self, outpoint: &(Txid, Vout)) -> Option<Txid> {
        self.map.get(outpoint).copied()
    }

    /// The live spender of `txid:vout` and the input spending it.
    pub fn spender<'a>(
        &self,
        txs: &'a ReadOnlyTxStore,
        txid: &Txid,
        vout: Vout,
    ) -> Option<(&'a TxRecord, Vin)> {
        let spender = txs.record(&self.get(&(*txid, vout))?)?;
        let vin = spender
            .tx
            .input
            .iter()
            .position(|input| input.txid == *txid && input.vout == vout)?;
        Some((spender, Vin::from(vin)))
    }

    /// The live spenders of `parent`'s outputs, each once: O(outputs).
    pub fn children<'a>(
        &self,
        txs: &'a ReadOnlyTxStore,
        parent: &'a Transaction,
    ) -> impl Iterator<Item = &'a TxRecord> {
        let mut spenders: SmallVec<[Txid; 4]> = (0..parent.output.len())
            .filter_map(|vout| self.get(&(parent.txid, Vout::from(vout))))
            .collect();
        spenders.sort_unstable_by_key(|txid| **txid);
        spenders.dedup();
        spenders.into_iter().filter_map(move |txid| txs.record(&txid))
    }

    /// Mempool txs are never coinbase: every input spends an outpoint.
    fn spent_outpoints(tx: &Transaction) -> impl Iterator<Item = (Txid, Vout)> + '_ {
        tx.input.iter().map(|input| (input.txid, input.vout))
    }
}
