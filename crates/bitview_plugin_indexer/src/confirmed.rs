use std::cell::OnceCell;

use bitcoin::{ScriptBuf, Transaction, Weight as BitcoinWeight, consensus::deserialize};
use bitview_primitives::{Lengths, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, OutputType, Sats, TxIndex, TxOut, Txid, TxidPrefix, Vout};
use rustc_hash::FxHashMap;
use vecdb::{ReadableVec, Ro};

use crate::{Indexer, UtxoKey};

/// Reads of confirmed transactions: their bytes from the block files, and the outputs they spend.
impl Indexer<Ro> {
    /// One indexed transaction's bytes and decoding. The caller's pin covers `index`.
    pub fn read_tx(&self, index: TxIndex) -> Result<(Vec<u8>, Transaction)> {
        let transactions = &self.vecs().transactions;
        let missing = || Error::Internal("Indexed transaction metadata missing");
        let size = *transactions
            .total_size
            .collect_one(index)
            .ok_or_else(missing)? as usize;
        let position = transactions
            .position
            .collect_one(index)
            .ok_or_else(missing)?;
        let txid = transactions.txid.collect_one(index).ok_or_else(missing)?;
        self.read_tx_at(position, size, txid)
    }

    /// One indexed transaction from metadata the caller read, without trusting persisted
    /// allocation lengths or accepting a different transaction at a stale/corrupt position.
    pub fn read_tx_at(
        &self,
        position: BlkPosition,
        size: usize,
        txid: Txid,
    ) -> Result<(Vec<u8>, Transaction)> {
        if size == 0 || size > BitcoinWeight::MAX_BLOCK.to_wu() as usize {
            return Err(Error::Internal(
                "Indexed transaction size exceeds block bounds",
            ));
        }
        let bytes = self.reader().read_raw_bytes(position, size)?;
        let transaction: Transaction = deserialize(&bytes)
            .map_err(|_| Error::Internal("Invalid indexed transaction encoding"))?;
        if Txid::from(transaction.compute_txid()) != txid
            || transaction.weight() > BitcoinWeight::MAX_BLOCK
        {
            return Err(Error::Internal(
                "Indexed transaction identity or weight mismatch",
            ));
        }
        Ok((bytes, transaction))
    }

    /// The confirmed outputs `holes` spend, as published at `safe` (the caller's pin); misses
    /// (unknown or unpublished parents, invalid outputs) are left out.
    pub fn prevouts(
        &self,
        holes: &[(Txid, Vout)],
        safe: Lengths,
    ) -> FxHashMap<(Txid, Vout), TxOut> {
        let mut resolved = FxHashMap::default();
        if holes.is_empty() {
            return resolved;
        }
        let vecs = self.vecs();
        let txid_reader = vecs.transactions.txid.reader();
        let first_txout_reader = vecs.transactions.first_txout_index.reader();
        let output_type_reader = vecs.outputs.output_type.reader();
        let type_index_reader = vecs.outputs.type_index.reader();
        let value_reader = vecs.outputs.value.reader();
        let addr_readers = vecs.addrs.addr_readers();
        // A confirmed unspent output with an address or empty script: a UTXO-store lookup (which
        // carries the value, but for a large one), the txid check and its address bytes.
        let unspent = |txid: Txid, vout: Vout| -> Option<TxOut> {
            let key = UtxoKey::new(TxidPrefix::from(txid), u32::from(vout));
            let utxo = self.stores().utxo(&key).ok()??;
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
            // A large value is a second lookup; spent in between, it misses like the entry.
            let value = self.stores().utxo_value(&key, &utxo).ok()?;
            Some(TxOut::from((script_pubkey, value)))
        };
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
        for (prev_txid, vouts) in parents {
            let Some(prev_tx_index) = self
                .stores()
                .tx_index(&TxidPrefix::from(prev_txid))
                .ok()
                .flatten()
            else {
                continue;
            };
            if prev_tx_index >= safe.tx_index
                || txid_reader.try_get(prev_tx_index) != Some(prev_txid)
            {
                continue;
            }
            let Some(first_txout) = first_txout_reader.try_get(prev_tx_index).map(|i| i.get())
            else {
                continue;
            };
            let next_tx = prev_tx_index.incremented();
            let next_txout = if next_tx < safe.tx_index {
                match first_txout_reader.try_get(next_tx) {
                    Some(next) => next.get(),
                    None => continue,
                }
            } else {
                safe.txout_index
            };
            if first_txout > next_txout || next_txout > safe.txout_index {
                continue;
            }
            // Decode at most one raw parent at a time. Retain only the requested outputs, not
            // every large parent in the batch.
            let decoded = OnceCell::new();
            resolved.extend(vouts.into_iter().filter_map(|vout| {
                let txout = usize::from(first_txout).checked_add(usize::from(vout))?;
                if txout >= usize::from(next_txout) {
                    return None;
                }
                let txout = TxOutIndex::from(txout);
                let output_type: OutputType = output_type_reader.try_get(txout)?;
                let type_index: TypeIndex = type_index_reader.try_get(txout)?;
                let value: Sats = value_reader.try_get(txout)?;
                if type_index >= safe.to_type_index(output_type) {
                    return None;
                }
                let script_pubkey = if output_type == OutputType::Empty {
                    ScriptBuf::new()
                } else if let Some(addr) = addr_readers.get(output_type, type_index) {
                    addr.to_script_pubkey()
                } else if matches!(
                    output_type,
                    OutputType::P2MS | OutputType::Unknown | OutputType::OpReturn
                ) {
                    let parent = decoded
                        .get_or_init(|| self.read_tx(prev_tx_index).ok().map(|(_, parent)| parent));
                    let output = parent.as_ref()?.output.get(usize::from(vout))?;
                    if output.value.to_sat() != u64::from(value) {
                        return None;
                    }
                    output.script_pubkey.clone()
                } else {
                    return None;
                };
                Some(((prev_txid, vout), TxOut::from((script_pubkey, value))))
            }));
        }
        resolved
    }
}
