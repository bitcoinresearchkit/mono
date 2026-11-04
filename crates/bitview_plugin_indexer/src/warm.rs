use std::sync::Arc;

use bitview_primitives::{AddrHash, TxOutIndex, TypeIndex};
use brk_types::{OutputType, Transaction, TxidPrefix};
use rayon::prelude::*;
use vecdb::StorageMode;

use crate::{Indexer, UtxoKey};

/// What indexing a block of transactions would hand the plugins after the indexer: the outputs
/// its inputs spend and the existing addresses it touches.
pub struct Touched {
    pub spent: Vec<TxOutIndex>,
    pub addresses: Vec<(OutputType, TypeIndex)>,
}

impl<M: StorageMode> Indexer<M> {
    /// Looks up what indexing `txs` would (each input's UTXO, each output's address), so a block
    /// holding them finds it in memory. Inputs spending each other and new addresses find
    /// nothing, as they would then.
    pub fn warm(&self, txs: &[Arc<Transaction>]) -> Touched {
        let stores = self.stores();
        let utxos: Vec<_> = txs
            .par_iter()
            .flat_map_iter(|tx| &tx.input)
            .filter_map(|txin| {
                let key = UtxoKey::new(TxidPrefix::from(&txin.txid), u32::from(txin.vout));
                let utxo = stores.utxos().get(&key).ok()??;
                // A large value is a second lookup, as resolving the input makes.
                let _ = stores.utxos().value(&key, &utxo);
                Some(utxo)
            })
            .collect();
        let paid: Vec<_> = txs
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
            })
            .collect();

        let mut addresses: Vec<_> = utxos
            .iter()
            .filter(|utxo| utxo.output_type().is_addr())
            .map(|utxo| (utxo.output_type(), utxo.type_index()))
            .chain(paid)
            .collect();
        // Each once and in order, as the update loads them.
        addresses.par_sort_unstable();
        addresses.dedup();
        Touched {
            spent: utxos.iter().map(|utxo| utxo.txout_index()).collect(),
            addresses,
        }
    }
}
