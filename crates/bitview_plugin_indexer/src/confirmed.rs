use bitcoin::{
    PubkeyHash, Script, ScriptBuf, ScriptHash, Transaction, TxIn, WPubkeyHash, WScriptHash, Weight,
    consensus::deserialize,
    hashes::{Hash, ripemd160},
};
use bitview_primitives::{Index40, Lengths, TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, OutputType, Sats, TxIndex, TxOut, Txid, TxidPrefix, Vout};
use rapidhash::RapidHashMap;
use sha2::{Digest, Sha256};
use vecdb::{BytesVecReader, ReadableVec, Ro, StorageMode};

use crate::{
    Indexer, UtxoKey, processor::transaction::analysis::policy::is_dust, readers::AddrReaders,
};

/// Reads of confirmed transactions: their bytes from the block files.
impl<M: StorageMode> Indexer<M> {
    /// One indexed transaction, decoded. The caller's pin covers `index`.
    pub fn read_tx(&self, index: TxIndex) -> Result<Transaction> {
        let (position, size) = self.tx_place(index)?;
        self.read_tx_at(position, size)
    }

    /// One indexed transaction's bytes as the block files hold them: nothing is decoded. The
    /// caller's pin covers `index`.
    pub fn read_tx_bytes(&self, index: TxIndex) -> Result<Vec<u8>> {
        let (position, size) = self.tx_place(index)?;
        self.read_tx_bytes_at(position, size)
    }

    /// One indexed transaction, decoded, from the position and size the caller read.
    fn read_tx_at(&self, position: BlkPosition, size: usize) -> Result<Transaction> {
        decode_tx(&self.read_tx_bytes_at(position, size)?)
    }

    /// Indexed transactions, decoded, from the positions and sizes the caller read: their bytes
    /// are read as one batch ([`brk_reader::Reader::read_raw_batch`]).
    pub fn read_txs_at(&self, places: &[(BlkPosition, usize)]) -> Result<Vec<Transaction>> {
        debug_assert!(
            places
                .iter()
                .all(|&(_, size)| size as u64 <= Weight::MAX_BLOCK.to_wu()),
            "a transaction larger than a block is a corrupt column"
        );
        self.reader()
            .read_raw_batch(places)?
            .iter()
            .map(|bytes| decode_tx(bytes))
            .collect()
    }

    /// Where the block files hold an indexed transaction, and how many bytes of it.
    fn tx_place(&self, index: TxIndex) -> Result<(BlkPosition, usize)> {
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
        Ok((position, size))
    }

    /// The index wrote the position and size from this node's block files, so the bytes there
    /// are the transaction: nothing is hashed to check it again.
    fn read_tx_bytes_at(&self, position: BlkPosition, size: usize) -> Result<Vec<u8>> {
        // Serialized bytes cannot exceed weight: a larger size is a corrupt column.
        debug_assert!(
            size as u64 <= Weight::MAX_BLOCK.to_wu(),
            "{size} bytes at {position:?}"
        );
        self.reader().read_raw_bytes(position, size)
    }

    /// The positions of a transaction's dust outputs, by the rule its `DUST_OUTPUT` flag follows.
    ///
    /// It reads the transaction from the block files, as the server's transaction reads do: the dust
    /// rule needs output scripts the indexer doesn't store for every type (bare multisig and unknown
    /// scripts, whose threshold depends on their length). It is called only for the rare fee-less
    /// transaction flagged with a dust output, so almost every block reads nothing; storing a dust
    /// position per transaction instead would cost a column over every transaction.
    pub fn dust_outputs(&self, index: TxIndex) -> Result<Vec<Vout>> {
        let transaction = self.read_tx(index)?;
        Ok(transaction
            .output
            .iter()
            .enumerate()
            .filter(|(_, output)| {
                let script = &output.script_pubkey;
                is_dust(output.value, OutputType::from(script), script)
            })
            .map(|(vout, _)| Vout::from(vout))
            .collect())
    }
}

/// Reads of confirmed transactions' outputs, and of the outputs they spend.
impl Indexer<Ro> {
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
    ) -> RapidHashMap<(Txid, Vout), TxOut> {
        let mut resolved = RapidHashMap::default();
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
        let mut parents: RapidHashMap<Txid, Vec<Vout>> = RapidHashMap::default();
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
    /// Starts reading both boundaries of each transaction in a sorted batch.
    pub fn prefetch(&self, transactions: impl IntoIterator<Item = TxIndex>) {
        self.first_txout.prefetch(
            transactions
                .into_iter()
                .flat_map(|tx| [tx, tx.incremented()])
                .filter(|&tx| tx < self.safe.tx_index),
        );
    }

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

/// The script of the output `input` spends, rebuilt from what the spend reveals, for the types
/// whose script commits to a hash consensus checks against the spend: the scriptSig's last push
/// for P2PKH (the key) and P2SH (the redeem script, whatever it holds: wrapped segwit,
/// multisig), witness item 1 for P2WPKH, the last witness item for P2WSH. Exact for every input
/// in the chain (3.57 billion, checked 2026-11-18), and a hash instead of a random read of the
/// address store. `None` for every other type (P2TR, P2A, P2PK, bare multisig, unknown) and for
/// a scriptSig that isn't push-only (`OP_1`-`OP_16` included) or a witness of another shape: the
/// caller reads those with [`OutputScripts::get`].
pub fn revealed_script(output_type: OutputType, input: &TxIn) -> Option<ScriptBuf> {
    let last_push = || input.script_sig.redeem_script().map(Script::as_bytes);
    let witness = &input.witness;
    match output_type {
        OutputType::P2PKH => Some(ScriptBuf::new_p2pkh(&PubkeyHash::from_byte_array(
            hash160(last_push()?),
        ))),
        OutputType::P2SH => Some(ScriptBuf::new_p2sh(&ScriptHash::from_byte_array(hash160(
            last_push()?,
        )))),
        OutputType::P2WPKH if witness.len() == 2 => Some(ScriptBuf::new_p2wpkh(
            &WPubkeyHash::from_byte_array(hash160(&witness[1])),
        )),
        OutputType::P2WSH => Some(ScriptBuf::new_p2wsh(&WScriptHash::from_byte_array(
            Sha256::digest(witness.last()?).into(),
        ))),
        _ => None,
    }
}

/// RIPEMD-160 of SHA-256, its SHA-256 through `sha2`, which uses the CPU's SHA instructions:
/// rust-bitcoin's hashes in software, which was ~3% of an address page's CPU.
fn hash160(bytes: &[u8]) -> [u8; 20] {
    ripemd160::Hash::hash(&Sha256::digest(bytes)).to_byte_array()
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
            slot => &mut slot.insert((tx_index, self.indexer.read_tx(tx_index)?)).1,
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

fn decode_tx(bytes: &[u8]) -> Result<Transaction> {
    deserialize(bytes).map_err(|_| Error::Internal("Invalid indexed transaction encoding"))
}
