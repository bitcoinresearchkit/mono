use crate::{ReadPin, check_bound};
use bitcoin::Transaction as BitcoinTransaction;
use bitview_plugin_indexer::{SafeLengths, revealed_script};
use bitview_primitives::{Bytes32, TxInIndex, TypeIndex};
use bitview_types::BlockTxIndex;
use brk_types::{
    BlkPosition, Height, OutPoint, OutputType, RawLockTime, Sats, SigOps32, Transaction, TxIn,
    TxIndex, TxOut, TxStatus, Txid, Vout, Weight,
};
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result};

impl Query {
    pub(crate) fn block_txs_at_height(
        &self,
        height: Height,
        start_index: BlockTxIndex,
        count: u32,
        guard: &ReadPin,
    ) -> Result<Vec<Transaction>> {
        let (first, tx_count) = self.block_tx_range(height, guard)?;
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
        self.transactions_at_indices(&indices, guard)
    }
    /// The txids of the block at `height`, in block order; errors as [`Self::block_tx_range`].
    pub(crate) fn block_txids_by_height(
        &self,
        height: Height,
        guard: &ReadPin,
    ) -> Result<Vec<Txid>> {
        let (first, tx_count) = self.block_tx_range(height, guard)?;
        let txids = self
            .indexer()
            .vecs()
            .transactions
            .txid
            .collect_range_at(first, first + tx_count);
        if txids.len() != tx_count {
            return Err(Error::Internal("Short block txid read"));
        }
        Ok(txids)
    }
    /// The txid at an in-block offset; `OutOfRange` past the block's last transaction, other
    /// errors as [`Self::block_tx_range`].
    pub(crate) fn block_txid_at_index_by_height(
        &self,
        height: Height,
        index: usize,
        guard: &ReadPin,
    ) -> Result<Txid> {
        let (first, tx_count) = self.block_tx_range(height, guard)?;
        if index >= tx_count {
            return Err(Error::OutOfRange("Transaction index out of range".into()));
        }
        self.indexer()
            .vecs()
            .transactions
            .txid
            .collect_one_at(first + index)
            .ok_or(Error::Internal("Block txid missing"))
    }
    /// Internal batch read; caller holds its pin from selection.
    pub(crate) fn transactions_at_indices(
        &self,
        indices: &[TxIndex],
        guard: &ReadPin,
    ) -> Result<Vec<Transaction>> {
        if indices.is_empty() {
            return Ok(Vec::new());
        }
        let safe = guard.lengths();
        if indices.iter().any(|index| *index >= safe.tx_index) {
            return Err(Error::UnknownTxid);
        }

        let len = indices.len();

        // Sort positions ascending for sequential I/O (O(n) when already sorted)
        let mut order: Vec<usize> = (0..len).collect();
        order.sort_unstable_by_key(|&i| indices[i]);

        let indexer = self.indexer();

        let txids = indexer.vecs().transactions.txid.reader();
        let mut total_size_cursor = indexer.vecs().transactions.total_size.cursor();
        let mut sigops_cursor = indexer.vecs().transactions.total_sigop_cost.cursor();
        let mut first_txin_cursor = indexer.vecs().transactions.first_txin_index.cursor();
        let mut position_cursor = indexer.vecs().transactions.position.cursor();

        struct DecodedTx {
            pos: usize,
            tx_index: TxIndex,
            txid: Txid,
            total_size: Bytes32,
            total_sigop_cost: SigOps32,
            status: TxStatus,
            decoded: BitcoinTransaction,
            first_txin_index: TxInIndex,
            /// Per input; `None` for the coinbase's.
            prevouts: Vec<Option<TxOut>>,
        }

        // One mapping for the request; the cursor answers a block's transactions in O(1).
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();

        let mut decoded_txs: Vec<DecodedTx> = Vec::with_capacity(len);
        let mut cached_status: Option<(Height, TxStatus)> = None;

        for &pos in &order {
            let tx_index = indices[pos];
            let idx = tx_index.to_usize();

            let txid: Txid = txids.try_get_at(idx).data()?;
            let total_size: Bytes32 = total_size_cursor.get(idx).data()?;
            let total_sigop_cost: SigOps32 = sigops_cursor.get(idx).data()?;
            let first_txin_index: TxInIndex = first_txin_cursor.get(idx).data()?;
            let position: BlkPosition = position_cursor.get(idx).data()?;

            // Every index is below `safe.tx_index` (checked above).
            let height: Height = tx_heights.get(tx_index).data()?;
            let status = if let Some((h, ref s)) = cached_status
                && h == height
            {
                s.clone()
            } else {
                let s = self.confirmed_status_at_bounded(height, safe)?;
                cached_status = Some((height, s.clone()));
                s
            };

            let decoded = indexer.read_tx_at(position, *total_size as usize)?;

            decoded_txs.push(DecodedTx {
                pos,
                tx_index,
                txid,
                total_size,
                total_sigop_cost,
                status,
                decoded,
                first_txin_index,
                prevouts: Vec::new(),
            });
        }

        // Sequential input cursors reuse decompressed pages, and reading output type, type index
        // and value from the input vecs avoids random reads of the output vecs.
        let mut input_output_type_cursor = indexer.vecs().inputs.output_type.cursor();
        let mut input_value_cursor = indexer.vecs().inputs.value.cursor();
        let mut outpoint_cursor = indexer.vecs().inputs.outpoint.cursor();
        let mut input_type_index_cursor = indexer.vecs().inputs.type_index.cursor();

        // A prevout's value is always read; its script is rebuilt from the spending input where
        // the spend reveals it, and only the rest read their outpoint, type index and script.
        let mut reads: Vec<(usize, usize, OutPoint, OutputType, TypeIndex, Sats)> = Vec::new();
        for (t, dtx) in decoded_txs.iter_mut().enumerate() {
            if dtx.decoded.is_coinbase() {
                dtx.prevouts = vec![None];
                continue;
            }
            let start = usize::from(dtx.first_txin_index);
            dtx.prevouts = dtx
                .decoded
                .input
                .iter()
                .enumerate()
                .map(|(j, txin)| {
                    let i = start + j;
                    let output_type: OutputType = input_output_type_cursor.get(i).data()?;
                    let value: Sats = input_value_cursor.get(i).data()?;
                    if let Some(script) = revealed_script(output_type, txin) {
                        return Ok(Some(TxOut::from((script, value))));
                    }
                    let outpoint: OutPoint = outpoint_cursor.get(i).data()?;
                    let type_index: TypeIndex = input_type_index_cursor.get(i).data()?;
                    reads.push((t, j, outpoint, output_type, type_index, value));
                    Ok(None)
                })
                .collect::<Result<_>>()?;
        }

        // Sorted by (output_type, type_index) for sequential BytesVec access within each address
        // type's file; non-address output indices are chronological within each script type, so
        // a decoded parent serves its adjacent outputs.
        reads.sort_unstable_by_key(|&(_, _, _, output_type, type_index, _)| {
            (output_type, type_index)
        });
        let mut scripts = indexer.output_scripts();
        for (t, j, outpoint, output_type, type_index, value) in reads {
            let script_pubkey = scripts.get(
                outpoint.tx_index(),
                outpoint.vout(),
                output_type,
                type_index,
                value,
            )?;
            decoded_txs[t].prevouts[j] = Some(TxOut::from((script_pubkey, value)));
        }
        drop(scripts);

        let mut txs: Vec<Option<Transaction>> = (0..len).map(|_| None).collect();

        for dtx in decoded_txs {
            let weight = Weight::from(dtx.decoded.weight());
            let is_coinbase = dtx.decoded.is_coinbase();
            let input: Vec<TxIn> = dtx
                .decoded
                .input
                .into_iter()
                .zip(dtx.prevouts)
                .map(|(txin, prevout)| {
                    // Every slot is filled by now, derived from the spend or read from the store; a failed read returned early.
                    debug_assert!(prevout.is_some() != is_coinbase, "prevout slot left unfilled");
                    let (txid, vout) = if is_coinbase {
                        (Txid::COINBASE, Vout::MAX)
                    } else {
                        (
                            Txid::from(txin.previous_output.txid),
                            Vout::from(txin.previous_output.vout),
                        )
                    };
                    TxIn {
                        txid,
                        vout,
                        prevout,
                        script_sig: txin.script_sig,
                        witness: txin.witness,
                        is_coinbase,
                        sequence: txin.sequence.0,
                    }
                })
                .collect();

            let output: Vec<TxOut> = dtx.decoded.output.into_iter().map(TxOut::from).collect();

            let mut transaction = Transaction {
                index: Some(dtx.tx_index),
                txid: dtx.txid,
                version: dtx.decoded.version.into(),
                lock_time: RawLockTime::from(dtx.decoded.lock_time),
                total_size: *dtx.total_size as usize,
                weight,
                total_sigop_cost: dtx.total_sigop_cost.into(),
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
    /// The block at `height`'s transactions as `(first, count)` in the flat tx vecs, under
    /// `pin`: the published tip block ends at `pin`'s transaction length, never at the writer's
    /// unpublished one. `OutOfRange` past the published tip; `Internal` for boundaries that
    /// are missing, empty or past the published data.
    pub(crate) fn block_tx_range(
        &self,
        height: Height,
        pin: &SafeLengths,
    ) -> Result<(usize, usize)> {
        let safe = pin.lengths();
        check_bound(pin, height, safe.height)?;
        if height >= safe.height {
            return Err(Error::OutOfRange("Block height out of range".into()));
        }
        let first_tx_index = &self.indexer().vecs().transactions.first_tx_index;
        let first = first_tx_index.collect_one(height).data()?;
        let next_height = height.incremented();
        let next = if next_height < safe.height {
            first_tx_index.collect_one(next_height).data()?
        } else {
            safe.tx_index
        };
        let count = Self::block_tx_count(first, next, safe.tx_index)?;
        Ok((first.to_usize(), count as usize))
    }
}
