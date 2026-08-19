use std::io::Cursor;

use bitcoin::consensus::Decodable;
use brk_error::{Error, OptionData};
use brk_types::{
    BlkPosition, BlockHash, BlockTxIndex, Height, OutPoint, OutputType, RawLockTime, Sats, SigOps,
    StoredU32, Transaction, TxIn, TxInIndex, TxIndex, TxOut, TxStatus, Txid, TypeIndex, Vout,
    Weight,
};
use rustc_hash::FxHashMap;
use vecdb::{ReadableVec, VecIndex};

use crate::Query;

impl Query {
    /// All txids in the block, canonical order (coinbase first).
    /// `NotFound` if the hash is unknown (or only collides on the 8-byte
    /// prefix), `OutOfRange` if the resolved height is past the indexed tip.
    /// Unpaginated by design.
    pub fn block_txids(&self, hash: &BlockHash) -> brk_error::Result<Vec<Txid>> {
        let height = self.height_by_hash(hash)?;
        self.block_txids_by_height(height)
    }

    /// Up to `count` transactions from the block, starting at the in-block
    /// offset `start_index` (0 = coinbase). `OutOfRange` when `start_index`
    /// is past the last tx in the block. Caller (route layer) sets `count`.
    pub fn block_txs(
        &self,
        hash: &BlockHash,
        start_index: BlockTxIndex,
        count: u32,
    ) -> brk_error::Result<Vec<Transaction>> {
        let height = self.height_by_hash(hash)?;
        let (first, tx_count) = self.block_tx_range(height)?;
        let start: usize = start_index.into();
        if start >= tx_count {
            return Err(Error::OutOfRange(
                "start index past last transaction in block".into(),
            ));
        }
        let count = (count as usize).min(tx_count - start);
        let indices: Vec<TxIndex> = (first + start..first + start + count)
            .map(TxIndex::from)
            .collect();
        self.transactions_by_indices(&indices)
    }

    /// Txid at an in-block offset (`index` is the position within the block,
    /// 0 = coinbase). `NotFound` if the hash is unknown or only collides on
    /// the 8-byte prefix; `OutOfRange` if `index` is past the last tx in
    /// the block.
    pub fn block_txid_at_index(
        &self,
        hash: &BlockHash,
        index: BlockTxIndex,
    ) -> brk_error::Result<Txid> {
        let height = self.height_by_hash(hash)?;
        self.block_txid_at_index_by_height(height, index.into())
    }

    // === Helper methods ===

    /// All txids in the block at `height`, canonical order. `OutOfRange`
    /// when `height` is past the indexed tip; `Internal` if any read hits
    /// the stamp-before-data race or short-returns. Used by both the
    /// hash-keyed and height-keyed entry points so they share bounds
    /// semantics.
    fn block_txids_by_height(&self, height: Height) -> brk_error::Result<Vec<Txid>> {
        let (first, tx_count) = self.block_tx_range(height)?;
        let txids = self
            .indexer()
            .vecs()
            .transactions
            .txid
            .collect_range_at(first, first + tx_count);
        if txids.len() != tx_count {
            return Err(Error::Internal("block_txids_by_height: short txid read"));
        }
        Ok(txids)
    }

    /// Single txid at an in-block offset. `OutOfRange` when `index` is past
    /// the last tx in the block. `Internal` if the underlying read finds
    /// the stamp-before-data race (`first_tx_index` flushed ahead of `txid`).
    fn block_txid_at_index_by_height(
        &self,
        height: Height,
        index: usize,
    ) -> brk_error::Result<Txid> {
        let (first, tx_count) = self.block_tx_range(height)?;
        if index >= tx_count {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        self.indexer()
            .vecs()
            .transactions
            .txid
            .collect_one_at(first + index)
            .ok_or(Error::Internal(
                "block_txid_at_index_by_height: txid index past data",
            ))
    }

    /// Batch-read transactions at arbitrary indices.
    /// Reads in ascending index order for I/O locality, returns in caller's order.
    ///
    /// Three-phase approach for sequential cursor I/O:
    ///   Phase 1: decode transactions, collect outpoints + per-input prevout
    ///            metadata (sorted by tx_index).
    ///   Phase 2: resolve each prevout's script_pubkey (sorted by
    ///            output_type, then type_index, for sequential addr-vec reads).
    ///   Phase 3: assemble `Transaction` objects, compute fees.
    ///
    /// The final `unwrap` is provably safe: `order` is a permutation of
    /// `0..len`, Phase 1 produces exactly one `DecodedTx` per position, and
    /// Phase 3 assigns each `txs[pos]` once before the collect.
    pub fn transactions_by_indices(
        &self,
        indices: &[TxIndex],
    ) -> brk_error::Result<Vec<Transaction>> {
        if indices.is_empty() {
            return Ok(Vec::new());
        }

        let len = indices.len();

        // Sort positions ascending for sequential I/O (O(n) when already sorted)
        let mut order: Vec<usize> = (0..len).collect();
        order.sort_unstable_by_key(|&i| indices[i]);

        let indexer = self.indexer();
        // BLK file reader, distinct from the vec cursors below.
        let reader = self.reader();

        // ── Phase 1: Decode all transactions, collect outpoints ─────────

        let txid_cursor = indexer.vecs().transactions.txid.reader().cursor();
        let mut total_size_cursor = indexer.vecs().transactions.total_size.cursor();
        let mut sigops_cursor = indexer.vecs().transactions.total_sigop_cost.cursor();
        let mut first_txin_cursor = indexer.vecs().transactions.first_txin_index.cursor();
        let mut position_cursor = indexer.vecs().transactions.position.cursor();

        struct DecodedTx {
            pos: usize,
            tx_index: TxIndex,
            txid: Txid,
            total_size: StoredU32,
            total_sigop_cost: SigOps,
            status: TxStatus,
            decoded: bitcoin::Transaction,
            first_txin_index: TxInIndex,
            outpoints: Vec<OutPoint>,
        }

        let mut decoded_txs: Vec<DecodedTx> = Vec::with_capacity(len);
        let mut total_inputs: usize = 0;
        let mut cached_status: Option<(Height, TxStatus)> = None;

        // Phase 1a: Read metadata + decode transactions (no outpoint reads yet)
        for &pos in &order {
            let tx_index = indices[pos];
            let idx = tx_index.to_usize();

            let txid: Txid = txid_cursor.get(idx).data()?;
            let total_size: StoredU32 = total_size_cursor.get(idx).data()?;
            let total_sigop_cost: SigOps = sigops_cursor.get(idx).data()?;
            let first_txin_index: TxInIndex = first_txin_cursor.get(idx).data()?;
            let position: BlkPosition = position_cursor.get(idx).data()?;

            let height = crate::r#impl::tx::confirmed_status_height(self, tx_index)?;
            let status = if let Some((h, ref s)) = cached_status
                && h == height
            {
                s.clone()
            } else {
                let s = crate::r#impl::tx::confirmed_status_at(self, height)?;
                cached_status = Some((height, s.clone()));
                s
            };

            let buffer = reader.read_raw_bytes(position, *total_size as usize)?;
            let decoded = bitcoin::Transaction::consensus_decode(&mut Cursor::new(buffer))
                .map_err(|_| Error::Parse("Failed to decode transaction".into()))?;

            total_inputs += decoded.input.len();

            decoded_txs.push(DecodedTx {
                pos,
                tx_index,
                txid,
                total_size,
                total_sigop_cost,
                status,
                decoded,
                first_txin_index,
                outpoints: Vec::new(),
            });
        }

        // Phase 1b: Batch-read outpoints + prevout data via cursors. PcoVec
        // sequential cursors avoid re-decompressing the same pages.
        // Reading output_type/type_index/value HERE from inputs vecs (sequential)
        // avoids random-reading them from outputs vecs in Phase 2.
        let mut outpoint_cursor = indexer.vecs().inputs.outpoint.cursor();
        let mut input_output_type_cursor = indexer.vecs().inputs.output_type.cursor();
        let mut input_type_index_cursor = indexer.vecs().inputs.type_index.cursor();
        let mut input_value_cursor = self.plugins().inputs.value.cursor();

        let mut prevout_input_data: FxHashMap<OutPoint, (OutputType, TypeIndex, Sats)> =
            FxHashMap::with_capacity_and_hasher(total_inputs, Default::default());

        for dtx in &mut decoded_txs {
            let start = usize::from(dtx.first_txin_index);
            let count = dtx.decoded.input.len();
            let mut outpoints = Vec::with_capacity(count);
            for i in 0..count {
                let op: OutPoint = outpoint_cursor.get(start + i).data()?;
                if op.is_not_coinbase() {
                    let ot: OutputType = input_output_type_cursor.get(start + i).data()?;
                    let ti: TypeIndex = input_type_index_cursor.get(start + i).data()?;
                    let val: Sats = input_value_cursor.get(start + i).data()?;
                    prevout_input_data.insert(op, (ot, ti, val));
                }
                outpoints.push(op);
            }
            dtx.outpoints = outpoints;
        }

        // ── Phase 2: Build prevout TxOut map (script_pubkey from addr vecs) ──
        // Sort by (output_type, type_index) for sequential BytesVec access
        // within each address type's file.

        let addr_readers = indexer.vecs().addrs.addr_readers();

        let mut sorted_prevouts: Vec<(OutPoint, OutputType, TypeIndex, Sats)> =
            Vec::with_capacity(prevout_input_data.len());
        for (&op, &(ot, ti, val)) in &prevout_input_data {
            sorted_prevouts.push((op, ot, ti, val));
        }
        sorted_prevouts.sort_unstable_by_key(|&(_, ot, ti, _)| (ot, ti));

        let mut prevout_map: FxHashMap<OutPoint, TxOut> =
            FxHashMap::with_capacity_and_hasher(sorted_prevouts.len(), Default::default());

        for &(op, output_type, type_index, value) in &sorted_prevouts {
            let script_pubkey = addr_readers.script_pubkey(output_type, type_index);
            prevout_map.insert(op, TxOut::from((script_pubkey, value)));
        }

        // ── Phase 3: Assemble Transaction objects ───────────────────────

        let mut txs: Vec<Option<Transaction>> = (0..len).map(|_| None).collect();

        for dtx in decoded_txs {
            let input: Vec<TxIn> = dtx
                .decoded
                .input
                .iter()
                .enumerate()
                .map(|(j, txin)| {
                    let outpoint = dtx.outpoints[j];
                    let is_coinbase = outpoint.is_coinbase();

                    let (prev_txid, prev_vout, prevout) = if is_coinbase {
                        (Txid::COINBASE, Vout::MAX, None)
                    } else {
                        let prev_txid = Txid::from(txin.previous_output.txid);
                        let prev_txout = prevout_map.get(&outpoint).data()?.clone();
                        (prev_txid, outpoint.vout(), Some(prev_txout))
                    };

                    let witness = txin.witness.clone().into();

                    Ok(TxIn {
                        txid: prev_txid,
                        vout: prev_vout,
                        prevout,
                        script_sig: txin.script_sig.clone(),
                        script_sig_asm: (),
                        witness,
                        is_coinbase,
                        sequence: txin.sequence.0,
                        inner_redeem_script_asm: (),
                        inner_witness_script_asm: (),
                    })
                })
                .collect::<brk_error::Result<_>>()?;

            let weight = Weight::from(dtx.decoded.weight());

            let output: Vec<TxOut> = dtx.decoded.output.into_iter().map(TxOut::from).collect();

            let mut transaction = Transaction {
                index: Some(dtx.tx_index),
                txid: dtx.txid,
                version: dtx.decoded.version.into(),
                lock_time: RawLockTime::from(dtx.decoded.lock_time),
                total_size: *dtx.total_size as usize,
                weight,
                total_sigop_cost: dtx.total_sigop_cost,
                fee: Sats::ZERO,
                input,
                output,
                status: dtx.status,
            };

            transaction.compute_fee();
            txs[dtx.pos] = Some(transaction);
        }

        Ok(txs.into_iter().map(Option::unwrap).collect())
    }

    /// Half-open `[first, first + tx_count)` window into the flat tx vecs
    /// for the block at `height`. Single source of truth for the four
    /// `block_*` callers in this file.
    ///
    /// `OutOfRange` when `height` is past the indexed-tip stamp.
    /// `Internal` when `first_tx_index[height]` is missing under the
    /// stamp-before-data race. The tip-of-safe block falls back to
    /// `safe.tx_index` (not live `txid.len()`, which can be ahead of the
    /// writer's stamped boundary mid-block).
    fn block_tx_range(&self, height: Height) -> brk_error::Result<(usize, usize)> {
        let safe = self.safe_lengths();
        if height >= safe.height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        let first_tx_index_vec = &self.indexer().vecs().transactions.first_tx_index;
        let first: usize = first_tx_index_vec.collect_one(height).data()?.into();
        let next_height = height.incremented();
        let next: usize = if next_height < safe.height {
            first_tx_index_vec.collect_one(next_height).data()?.into()
        } else {
            safe.tx_index.to_usize()
        };
        Ok((first, next - first))
    }
}

#[inline]
pub fn block_txids_by_height(query: &Query, height: Height) -> brk_error::Result<Vec<Txid>> {
    query.block_txids_by_height(height)
}
