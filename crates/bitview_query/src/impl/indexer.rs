use brk_types::{OutputType, Sats, TxOut, TxOutIndex, Txid, TxidPrefix, TypeIndex, Vout};
use rustc_hash::FxHashMap;

use crate::Query;

impl Query {
    /// Indexer-backed resolver for confirmed-parent prevouts.
    pub fn indexer_prevout_resolver(
        &self,
    ) -> impl Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send + Sync + use<> {
        let indexer = self.0.plugins.indexer;

        move |holes: &[(Txid, Vout)]| {
            if holes.is_empty() {
                return FxHashMap::default();
            }
            let safe = indexer.safe_lengths();
            let first_txout_reader = indexer.vecs().transactions.first_txout_index.reader();
            let output_type_reader = indexer.vecs().outputs.output_type.reader();
            let type_index_reader = indexer.vecs().outputs.type_index.reader();
            let value_reader = indexer.vecs().outputs.value.reader();
            let addr_readers = indexer.vecs().addrs.addr_readers();
            holes
                .iter()
                .filter_map(|(prev_txid, vout)| {
                    let prev_tx_index = indexer
                        .stores()
                        .tx_index(&TxidPrefix::from(prev_txid))
                        .ok()??;
                    if prev_tx_index >= safe.tx_index {
                        return None;
                    }
                    let first_txout: TxOutIndex = first_txout_reader.try_get(prev_tx_index)?;
                    let txout = first_txout + *vout;
                    if txout >= safe.txout_index {
                        return None;
                    }
                    let output_type: OutputType = output_type_reader.try_get(txout)?;
                    let type_index: TypeIndex = type_index_reader.try_get(txout)?;
                    let value: Sats = value_reader.try_get(txout)?;
                    let script_pubkey = addr_readers.script_pubkey(output_type, type_index);
                    Some(((*prev_txid, *vout), TxOut::from((script_pubkey, value))))
                })
                .collect()
        }
    }
}
