use crate::{ReadPin, block_span, check_bound};
use bitcoin::Transaction as BitcoinTransaction;
use bitview_plugin_indexer::{SafeLengths, revealed_script};
use bitview_primitives::{Bytes32, TxInIndex, TypeIndex};
use bitview_types::BlockTxIndex;
use brk_types::{
    Height, OutPoint, OutputType, RawLockTime, Sats, SigOps32, Transaction, TxIn, TxIndex, TxOut,
    TxStatus, Txid, Vout, Weight,
};
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result, read_sorted};

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

        // The transactions are scattered over years of blocks. Each round reads its columns with
        // `read_sorted`, which asks for a column's pages together before reading any, so the
        // device serves them as one batch rather than one fault at a time. Round 1: the
        // transactions' columns, and their blocks' hashes and times.
        let sorted: Vec<TxIndex> = order.iter().map(|&pos| indices[pos]).collect();
        let rows: Vec<usize> = sorted.iter().map(|tx| tx.to_usize()).collect();
        let columns = &indexer.vecs().transactions;
        let txids = read_sorted(&columns.txid, &rows)?;
        let sizes = read_sorted(&*columns.total_size, &rows)?;
        let sigops = read_sorted(&*columns.total_sigop_cost, &rows)?;
        let first_inputs = read_sorted(&columns.first_txin_index, &rows)?;
        let positions = read_sorted(&columns.position, &rows)?;

        // One mapping for the request; the cursor answers a block's transactions in O(1).
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        // Every index is below `safe.tx_index` (checked above).
        let heights = sorted
            .iter()
            .map(|&tx| tx_heights.get(tx).data())
            .collect::<Result<Vec<Height>>>()?;
        let statuses = self.confirmed_statuses_bounded(&heights, safe)?;

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

        let mut decoded_txs: Vec<DecodedTx> = Vec::with_capacity(len);
        for (k, ((&pos, &tx_index), status)) in order.iter().zip(&sorted).zip(statuses).enumerate()
        {
            let decoded = indexer.read_tx_at(positions[k], *sizes[k] as usize)?;
            decoded_txs.push(DecodedTx {
                pos,
                tx_index,
                txid: txids[k],
                total_size: sizes[k],
                total_sigop_cost: sigops[k],
                status,
                decoded,
                first_txin_index: first_inputs[k],
                prevouts: Vec::new(),
            });
        }

        // Round 2: every input's output type and value, read from the input vecs rather than at
        // random in the output vecs. A coinbase has no prevout to read.
        let inputs = &indexer.vecs().inputs;
        let input_rows: Vec<usize> = decoded_txs
            .iter()
            .filter(|dtx| !dtx.decoded.is_coinbase())
            .flat_map(|dtx| {
                let start = usize::from(dtx.first_txin_index);
                start..start + dtx.decoded.input.len()
            })
            .collect();
        let output_types = read_sorted(&inputs.output_type, &input_rows)?;
        let values = read_sorted(&*inputs.value, &input_rows)?;

        // A prevout's script is rebuilt from the spending input where the spend reveals it; the
        // rest are read in round 3: (tx, input, position in `input_rows`).
        let mut unrevealed: Vec<(usize, usize, usize)> = Vec::new();
        let mut at = 0;
        for (t, dtx) in decoded_txs.iter_mut().enumerate() {
            if dtx.decoded.is_coinbase() {
                dtx.prevouts = vec![None];
                continue;
            }
            dtx.prevouts = dtx
                .decoded
                .input
                .iter()
                .enumerate()
                .map(|(j, txin)| {
                    let k = at + j;
                    let script = revealed_script(output_types[k], txin);
                    if script.is_none() {
                        unrevealed.push((t, j, k));
                    }
                    script.map(|script| TxOut::from((script, values[k])))
                })
                .collect();
            at += dtx.decoded.input.len();
        }

        // Round 3: the unrevealed prevouts' outpoints and type indexes.
        let rows: Vec<usize> = unrevealed.iter().map(|&(_, _, k)| input_rows[k]).collect();
        let outpoints = read_sorted(&*inputs.outpoint, &rows)?;
        let type_indexes = read_sorted(&*inputs.type_index, &rows)?;
        let mut reads: Vec<(usize, usize, OutPoint, OutputType, TypeIndex, Sats)> = unrevealed
            .into_iter()
            .zip(outpoints.into_iter().zip(type_indexes))
            .map(|((t, j, k), (outpoint, type_index))| {
                (t, j, outpoint, output_types[k], type_index, values[k])
            })
            .collect();

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
                    // A failed store read returned early, so every slot is filled.
                    debug_assert!(
                        prevout.is_some() != is_coinbase,
                        "prevout slot left unfilled"
                    );
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
        let txs = block_span(
            &self.indexer().vecs().transactions.first_tx_index,
            height,
            safe,
            safe.tx_index.to_usize(),
        )?;
        let count = Self::block_tx_count(
            TxIndex::from(txs.start),
            TxIndex::from(txs.end),
            safe.tx_index,
        )?;
        Ok((txs.start, count as usize))
    }
}
