use bitcoin::{ScriptBuf, Transaction, Weight as BitcoinWeight, consensus::deserialize};
use bitview_primitives::{Index40, Lengths, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, OutputType, Sats, TxIndex, TxOut, Txid, TxidPrefix, Vout};
use rustc_hash::FxHashMap;
use vecdb::{BytesVecReader, ReadableVec, Ro};

use crate::{Indexer, UtxoKey, readers::AddrReaders};

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

    /// Transaction output ranges as published at `safe`.
    pub fn output_ranges(&self, safe: Lengths) -> OutputRanges {
        OutputRanges {
            first_txout: self.vecs().transactions.first_txout_index.reader(),
            safe,
        }
    }

    pub fn output_scripts(&self) -> OutputScripts<'_> {
        OutputScripts {
            indexer: self,
            addrs: self.vecs().addrs.addr_readers(),
            parent: None,
        }
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
        let output_type_reader = vecs.outputs.output_type.reader();
        let type_index_reader = vecs.outputs.type_index.reader();
        let value_reader = vecs.outputs.value.reader();
        let ranges = self.output_ranges(safe);
        let mut scripts = self.output_scripts();
        // A confirmed unspent output with an address or empty script: a UTXO-store lookup (which
        // carries the value, but for a large one) and the txid check.
        let unspent = |scripts: &mut OutputScripts, txid: Txid, vout: Vout| -> Option<TxOut> {
            let key = UtxoKey::new(TxidPrefix::from(txid), u32::from(vout));
            let utxo = self.stores().utxos().get(&key).ok()??;
            let (tx_index, txout, output_type, type_index) = (
                utxo.tx_index(),
                utxo.txout_index(),
                utxo.output_type(),
                utxo.type_index(),
            );
            if script_in_parent(output_type)
                || tx_index >= safe.tx_index
                || txout >= safe.txout_index
                || type_index >= safe.to_type_index(output_type)
                || txid_reader.try_get(tx_index)? != txid
            {
                return None;
            }
            // A large value is a second lookup; spent in between, it misses like the entry.
            let value = self.stores().utxos().value(&key, &utxo).ok()?;
            let script_pubkey = scripts
                .get(tx_index, vout, output_type, type_index, value)
                .ok()?;
            Some(TxOut::from((script_pubkey, value)))
        };
        // The rest (unknown, unsafe or spent-above-safe parents; scripts kept only in the parent)
        // go through the parent transaction's indexed outputs, grouped so each parent decodes
        // once.
        let mut parents: FxHashMap<Txid, Vec<Vout>> = FxHashMap::default();
        for &(txid, vout) in holes {
            match unspent(&mut scripts, txid, vout) {
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
            let Some(outputs) = ranges.get(prev_tx_index) else {
                continue;
            };
            resolved.extend(vouts.into_iter().filter_map(|vout| {
                let txout = outputs.get(vout)?;
                let output_type: OutputType = output_type_reader.try_get(txout)?;
                let type_index: TypeIndex = type_index_reader.try_get(txout)?;
                let value: Sats = value_reader.try_get(txout)?;
                if type_index >= safe.to_type_index(output_type) {
                    return None;
                }
                let script_pubkey = scripts
                    .get(prev_tx_index, vout, output_type, type_index, value)
                    .ok()?;
                Some(((prev_txid, vout), TxOut::from((script_pubkey, value))))
            }));
        }
        resolved
    }
}

/// A transaction's outputs: `len` of them from `first`.
#[derive(Debug, Clone, Copy)]
pub struct OutputRange {
    pub first: TxOutIndex,
    pub len: usize,
}

impl OutputRange {
    /// Output `vout`, `None` past the transaction's outputs.
    #[inline]
    pub fn get(self, vout: Vout) -> Option<TxOutIndex> {
        (usize::from(vout) < self.len).then(|| self.first + vout)
    }
}

/// Transaction output ranges as published at a pin: a transaction's outputs run from its first
/// output to the next transaction's, the published output length for the last one.
pub struct OutputRanges {
    first_txout: BytesVecReader<TxIndex, Index40<TxOutIndex>>,
    safe: Lengths,
}

impl OutputRanges {
    /// `None` for an unpublished transaction or boundaries outside the published outputs.
    pub fn get(&self, tx_index: TxIndex) -> Option<OutputRange> {
        if tx_index >= self.safe.tx_index {
            return None;
        }
        let first = self.first_txout.try_get(tx_index)?.get();
        let next_tx = tx_index.incremented();
        let next = if next_tx < self.safe.tx_index {
            self.first_txout.try_get(next_tx)?.get()
        } else {
            self.safe.txout_index
        };
        let len = usize::from(next).checked_sub(usize::from(first))?;
        (next <= self.safe.txout_index).then_some(OutputRange { first, len })
    }
}

/// The output types whose scripts the indexer does not keep: read from their transaction.
fn script_in_parent(output_type: OutputType) -> bool {
    matches!(
        output_type,
        OutputType::P2MS | OutputType::UnknownScript | OutputType::OpReturn
    )
}

/// Rebuilds indexed outputs' scripts: empty, from their address bytes, or from their transaction
/// for the types whose scripts the indexer does not keep (P2MS, unknown, OP_RETURN), the last
/// one decoded kept for its next outputs.
pub struct OutputScripts<'a> {
    indexer: &'a Indexer<Ro>,
    addrs: AddrReaders,
    parent: Option<(TxIndex, Transaction)>,
}

impl OutputScripts<'_> {
    pub fn get(
        &mut self,
        tx_index: TxIndex,
        vout: Vout,
        output_type: OutputType,
        type_index: TypeIndex,
        value: Sats,
    ) -> Result<ScriptBuf> {
        if output_type == OutputType::EmptyScript {
            return Ok(ScriptBuf::new());
        }
        if let Some(addr) = self.addrs.get(output_type, type_index) {
            return Ok(addr.to_script_pubkey());
        }
        if !script_in_parent(output_type) {
            return Err(Error::Internal("Indexed output address missing"));
        }
        let parent = match &mut self.parent {
            Some((index, parent)) if *index == tx_index => parent,
            slot => &mut slot.insert((tx_index, self.indexer.read_tx(tx_index)?.1)).1,
        };
        parent
            .output
            .get(usize::from(vout))
            .filter(|output| output.value.to_sat() == u64::from(value))
            .map(|output| output.script_pubkey.clone())
            .ok_or(Error::Internal(
                "Indexed output missing from its transaction",
            ))
    }
}
