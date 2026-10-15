use std::collections::hash_map::Entry;

use brk_types::{Transaction, Txid, Vin, Vout};
use rustc_hash::FxHashMap;

use crate::{
    key::{OutpointKey, TxKey},
    stores::{ReadOnlyTxStore, tx_store::TxRecord},
};

/// Mempool index from spent outpoint to spending mempool tx.
///
/// Keys are `OutpointKey` (keyed txid hash + vout). Key collisions
/// are negligible but possible, so callers must verify the candidate
/// spender's input list. Values are the spender's `TxKey`,
/// looked up against `TxStore` to recover the full spender record.
#[derive(Clone, Default)]
pub struct OutpointSpends {
    map: FxHashMap<OutpointKey, TxKey>,
    /// Bumped on every mapping change, so a change of owner alone (no
    /// membership change) still republishes.
    revision: u64,
}

impl OutpointSpends {
    /// `replace` takes over outpoints already spent by another tx; otherwise
    /// only free ones are claimed.
    pub fn insert_spends(&mut self, tx: &Transaction, spender: TxKey, replace: bool) {
        let mut changed = false;
        for key in Self::spent_outpoints(tx) {
            changed |= if replace {
                self.map.insert(key, spender) != Some(spender)
            } else {
                match self.map.entry(key) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(spender);
                        true
                    }
                    Entry::Occupied(_) => false,
                }
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
        self.spender_if(txs, txid, vout, |_| true)
    }

    /// [`Self::spender`] when the stored key passes `accept`, checked before
    /// the input scan.
    pub fn spender_if<'a>(
        &self,
        txs: &'a ReadOnlyTxStore,
        txid: &Txid,
        vout: Vout,
        accept: impl FnOnce(TxKey) -> bool,
    ) -> Option<(&'a TxRecord, Vin)> {
        let key = self
            .get(&OutpointKey::new(TxKey::from(txid), vout))
            .filter(|key| accept(*key))?;
        let spender = txs.record_by_key(&key)?;
        let vin = spender
            .tx
            .input
            .iter()
            .position(|input| input.txid == *txid && input.vout == vout)?;
        Some((spender, Vin::from(vin)))
    }

    fn spent_outpoints(tx: &Transaction) -> impl Iterator<Item = OutpointKey> + '_ {
        tx.input
            .iter()
            .filter(|i| !i.is_coinbase)
            .map(|i| OutpointKey::new(TxKey::from(&i.txid), i.vout))
    }
}
