use std::cell::OnceCell;

use bitcoin::ScriptBuf;
use bitview_plugin_indexer::UtxoKey;
use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutputType, Sats, TxOut, Txid, TxidPrefix, Vout};
use rustc_hash::FxHashMap;

use super::indexed_transaction;
use crate::Query;

impl Query {
    /// Indexer-backed resolver for confirmed-parent prevouts.
    pub fn indexer_prevout_resolver(
        &self,
    ) -> impl Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send + Sync + use<> {
        let query = self.clone();

        move |holes: &[(Txid, Vout)]| {
            if holes.is_empty() {
                return FxHashMap::default();
            }
            let indexer = query.indexer();
            let Ok(pin) = query.pin_safe_lengths() else {
                return FxHashMap::default();
            };
            let safe = pin.lengths();
            let txid_reader = indexer.vecs().transactions.txid.reader();
            let first_txout_reader = indexer.vecs().transactions.first_txout_index.reader();
            let output_type_reader = indexer.vecs().outputs.output_type.reader();
            let type_index_reader = indexer.vecs().outputs.type_index.reader();
            let value_reader = indexer.vecs().outputs.value.reader();
            let addr_readers = indexer.vecs().addrs.addr_readers();
            // A confirmed unspent output with an address or empty script: a UTXO-store lookup, the txid
            // check, its value and address bytes.
            let unspent = |txid: Txid, vout: Vout| -> Option<TxOut> {
                let utxo = indexer
                    .stores()
                    .utxo(&UtxoKey::new(TxidPrefix::from(txid), vout))
                    .ok()??;
                let (tx_index, txout, output_type, type_index) = (
                    utxo.tx_index(),
                    utxo.txout_index(),
                    utxo.output_type(),
                    utxo.type_index(),
                );
                if tx_index >= safe.tx_index
                    || txout >= safe.txout_index
                    || type_index >= safe.to_type_index(output_type)
                    || txid_reader.try_get(tx_index)? != txid
                {
                    return None;
                }
                let script_pubkey = if output_type == OutputType::Empty {
                    ScriptBuf::new()
                } else {
                    addr_readers
                        .get(output_type, type_index)?
                        .to_script_pubkey()
                };
                Some(TxOut::from((script_pubkey, value_reader.try_get(txout)?)))
            };
            let mut resolved = FxHashMap::default();
            // The rest (misses: unknown, unsafe or spent-above-safe parents; P2MS and unknown scripts) go
            // through the parent transaction.
            let mut parents: FxHashMap<Txid, Vec<Vout>> = FxHashMap::default();
            for &(txid, vout) in holes {
                match unspent(txid, vout) {
                    Some(txout) => {
                        resolved.insert((txid, vout), txout);
                    }
                    None => parents.entry(txid).or_default().push(vout),
                }
            }
            resolved.extend(
                parents
                    .into_iter()
                    .filter_map(|(prev_txid, vouts)| {
                        let prev_tx_index = indexer
                            .stores()
                            .tx_index(&TxidPrefix::from(prev_txid))
                            .ok()??;
                        if prev_tx_index >= safe.tx_index
                            || txid_reader.try_get(prev_tx_index)? != prev_txid
                        {
                            return None;
                        }
                        let first_txout: TxOutIndex = first_txout_reader.try_get(prev_tx_index)?;
                        let next_tx = prev_tx_index.incremented();
                        let next_txout = if next_tx < safe.tx_index {
                            first_txout_reader.try_get(next_tx)?
                        } else {
                            safe.txout_index
                        };
                        if first_txout > next_txout || next_txout > safe.txout_index {
                            return None;
                        }
                        // Decode at most one raw parent at a time. Retain only the
                        // requested outputs, not every large parent in the batch.
                        let decoded = OnceCell::new();
                        Some(
                            vouts
                                .into_iter()
                                .filter_map(|vout| {
                                    let txout =
                                        usize::from(first_txout).checked_add(usize::from(vout))?;
                                    if txout >= usize::from(next_txout) {
                                        return None;
                                    }
                                    let txout = TxOutIndex::from(txout);
                                    let output_type: OutputType =
                                        output_type_reader.try_get(txout)?;
                                    let type_index: TypeIndex = type_index_reader.try_get(txout)?;
                                    let value: Sats = value_reader.try_get(txout)?;
                                    if type_index >= safe.to_type_index(output_type) {
                                        return None;
                                    }
                                    let script_pubkey = if output_type == OutputType::Empty {
                                        ScriptBuf::new()
                                    } else if let Some(addr) =
                                        addr_readers.get(output_type, type_index)
                                    {
                                        addr.to_script_pubkey()
                                    } else if matches!(
                                        output_type,
                                        OutputType::P2MS
                                            | OutputType::Unknown
                                            | OutputType::OpReturn
                                    ) {
                                        let parent = decoded.get_or_init(|| {
                                            indexed_transaction::read_at(
                                                &query,
                                                prev_tx_index,
                                                safe,
                                            )
                                            .ok()
                                            .map(|(_, parent)| parent)
                                        });
                                        let output =
                                            parent.as_ref()?.output.get(usize::from(vout))?;
                                        if output.value.to_sat() != u64::from(value) {
                                            return None;
                                        }
                                        output.script_pubkey.clone()
                                    } else {
                                        return None;
                                    };
                                    Some(((prev_txid, vout), TxOut::from((script_pubkey, value))))
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .flatten(),
            );
            resolved
        }
    }
}
