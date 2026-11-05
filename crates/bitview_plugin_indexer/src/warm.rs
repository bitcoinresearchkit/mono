use std::sync::Arc;

use bitview_primitives::{AddrHash, TypeIndex};
use brk_types::{OutputType, Transaction, TxidPrefix};
use rayon::prelude::*;
use vecdb::StorageMode;

use crate::{Indexer, UtxoKey};

impl<M: StorageMode> Indexer<M> {
    /// Looks up what indexing `txs` would (each input's UTXO, each output's address), so a block
    /// holding them finds it in memory, and returns the existing addresses it touches. Inputs
    /// spending each other and new addresses find nothing, as they would then.
    pub fn warm(&self, txs: &[Arc<Transaction>]) -> Vec<(OutputType, TypeIndex)> {
        let stores = self.stores();
        let spent = txs
            .par_iter()
            .flat_map_iter(|tx| &tx.input)
            .filter_map(|txin| {
                let key = UtxoKey::new(TxidPrefix::from(&txin.txid), u32::from(txin.vout));
                let utxo = stores.utxos().get(&key).ok()??;
                // A large value is a second lookup, as resolving the input makes.
                let _ = stores.utxos().value(&key, &utxo);
                let output_type = utxo.output_type();
                output_type
                    .is_addr()
                    .then(|| (output_type, utxo.type_index()))
            });
        let paid = txs
            .par_iter()
            .flat_map_iter(|tx| &tx.output)
            .filter_map(|txout| {
                let script = &txout.script_pubkey;
                let output_type = OutputType::from(script);
                if !output_type.is_addr() {
                    return None;
                }
                let hash = AddrHash::from_script(script, output_type).ok()?;
                Some((output_type, stores.addr_index(output_type, &hash).ok()??))
            });

        let mut addresses: Vec<_> = spent.chain(paid).collect();
        // Each once and in order, as the update loads them.
        addresses.par_sort_unstable();
        addresses.dedup();
        addresses
    }
}
