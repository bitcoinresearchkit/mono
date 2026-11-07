use brk_types::{Transaction, Txid, Vin, Vout};
use rustc_hash::FxBuildHasher;

use crate::{
    key::{OutpointKey, TxKey},
    stores::{ReadOnlyTxStore, cow::CowMap, tx_store::TxRecord},
};

/// Mempool index from spent outpoint to spending mempool tx.
///
/// Keys are `OutpointKey` (keyed txid hash + vout). Key collisions
/// are negligible but possible, so callers must verify the candidate
/// spender's input list. Values are the spender's `TxKey`,
/// looked up against `TxStore` to recover the full spender record.
#[derive(Clone, Default)]
pub struct OutpointSpends {
    map: CowMap<OutpointKey, TxKey, FxBuildHasher>,
    /// Bumped on every mapping change, so a change of owner alone (no
    /// membership change) still republishes.
    revision: u64,
}

impl OutpointSpends {
    /// `spender` owns the outpoints `tx` spends.
    pub fn insert_spends(&mut self, tx: &Transaction, spender: TxKey) {
        let mut changed = false;
        for key in Self::spent_outpoints(tx) {
            changed |= self.map.get(&key) != Some(&spender) && {
                self.map.insert(key, spender);
                true
            };
        }
        self.bump_if(changed);
    }

    /// Only removes entries whose stored key still matches `spender`,
    /// so an outpoint already re-claimed by a later spender is left alone.
    pub fn remove_spends(&mut self, tx: &Transaction, spender: TxKey) {
        let mut changed = false;
        for key in Self::spent_outpoints(tx) {
            if self.map.get(&key) == Some(&spender) {
                self.map.remove(&key);
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

    #[inline]
    pub fn get(&self, key: &OutpointKey) -> Option<TxKey> {
        self.map.get(key).copied()
    }

    /// The live spender of `txid:vout` and the input spending it, verified
    /// against the spender's full input list.
    pub fn spender<'a>(
        &self,
        txs: &'a ReadOnlyTxStore,
        txid: &Txid,
        vout: Vout,
    ) -> Option<(&'a TxRecord, Vin)> {
        let key = self.get(&OutpointKey::new(TxKey::from(txid), vout))?;
        let spender = txs.record_by_key(&key)?;
        let vin = spender
            .tx
            .input
            .iter()
            .position(|input| input.txid == *txid && input.vout == vout)?;
        Some((spender, Vin::from(vin)))
    }

    /// The live spenders of `tx`'s outputs, each once, verified once against its inputs:
    /// O(outputs + their inputs), however many of `tx`'s outputs one spends.
    pub fn children<'a>(&self, txs: &'a ReadOnlyTxStore, tx: &Transaction) -> Vec<&'a TxRecord> {
        let parent = TxKey::from(&tx.txid);
        let mut keys: Vec<TxKey> = (0..tx.output.len())
            .filter_map(|vout| self.get(&OutpointKey::new(parent, Vout::from(vout))))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        keys.iter()
            .filter_map(|key| txs.record_by_key(key))
            .filter(|child| child.tx.input.iter().any(|input| input.txid == tx.txid))
            .collect()
    }

    /// Mempool txs are never coinbase: every input spends an outpoint.
    fn spent_outpoints(tx: &Transaction) -> impl Iterator<Item = OutpointKey> + '_ {
        tx.input
            .iter()
            .map(|i| OutpointKey::new(TxKey::from(&i.txid), i.vout))
    }
}
