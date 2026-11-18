use std::slice::from_ref;

use bitview_plugin_outputs::published_spend;
use bitview_primitives::{Lengths, TxInIndex, TxOutIndex};
use brk_mempool::ReadOnlyState;
use brk_types::{BlockHash, Height, Timestamp, TxIndex, TxOutspend, TxStatus, Txid, Vin, Vout};
use serde_json::to_vec;
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, RepresentationId, Result};

impl Query {
    fn outspend(
        &self,
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
        txid: &Txid,
        vout: Vout,
    ) -> Result<TxOutspend> {
        let safe = pin.lengths();
        let outputs = match self.resolve_tx_outputs(txid, pin) {
            Ok(outputs) => outputs,
            Err(error) if Self::pool_answers(&error, pin) => {
                let error = match self.pending_view(mempool, pin)? {
                    Some((mempool, view)) => match mempool.outspend_if_present(txid, vout, &view)? {
                        Some(outspend) => return Ok(outspend),
                        None => Self::unanswered(error, &view, txid),
                    },
                    None => error,
                };
                // An output no known transaction holds is unspent, as Esplora answers.
                return match error {
                    Error::UnknownTxid => Ok(TxOutspend::UNSPENT),
                    error => Err(error),
                };
            }
            Err(error) => return Err(error),
        };
        let Some(txout) = outputs.get(vout) else {
            return Ok(TxOutspend::UNSPENT);
        };
        let confirmed = self.resolve_outspends(txout, 1, safe)?.pop().data()?;
        if confirmed.spent {
            return Ok(confirmed);
        }
        // Confirmed spends answer first; the pool, as the pin sees it, adds a pending spend of the
        // rest.
        Ok(match self.pending_view(mempool, pin)? {
            Some((mempool, view)) => mempool.outspend(txid, vout, &view)?,
            None => TxOutspend::UNSPENT,
        })
    }

    /// One outspend, serialized once, with its [`outspends_identity`].
    pub fn outspend_json(
        &self,
        txid: &Txid,
        vout: Vout,
    ) -> Result<(Vec<u8>, RepresentationId, bool)> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let outspend = self.outspend(mempool.as_deref(), &pin, txid, vout)?;
        let bytes = to_vec(&outspend).unwrap();
        let (identity, settled) = outspends_identity(from_ref(&outspend), &bytes, pin.lengths());
        Ok((bytes, identity, settled))
    }

    fn outspends(
        &self,
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
        txid: &Txid,
    ) -> Result<Vec<TxOutspend>> {
        let safe = pin.lengths();
        let outputs = match self.resolve_tx_outputs(txid, pin) {
            Ok(outputs) => outputs,
            Err(error) if Self::pool_answers(&error, pin) => {
                let Some((mempool, view)) = self.pending_view(mempool, pin)? else {
                    return Err(error);
                };
                return match mempool.outspends_if_present(txid, &view)? {
                    Some(outspends) => Ok(outspends),
                    None => Err(Self::unanswered(error, &view, txid)),
                };
            }
            Err(error) => return Err(error),
        };
        let mut outspends = self.resolve_outspends(outputs.first, outputs.len, safe)?;
        // Only what the chain leaves unspent asks the pool.
        if outspends.iter().any(|outspend| !outspend.spent)
            && let Some((mempool, view)) = self.pending_view(mempool, pin)?
        {
            mempool.merge_outspends(txid, &mut outspends, &view)?;
        }
        Ok(outspends)
    }

    /// A transaction's outspends, serialized once, with their [`outspends_identity`].
    pub fn outspends_json(&self, txid: &Txid) -> Result<(Vec<u8>, RepresentationId, bool)> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let outspends = self.outspends(mempool.as_deref(), &pin, txid)?;
        let bytes = to_vec(&outspends).unwrap();
        let (identity, settled) = outspends_identity(&outspends, &bytes, pin.lengths());
        Ok((bytes, identity, settled))
    }

    /// Resolve spend status for a contiguous range of outputs, as of the published `bound`: the
    /// spend index changes in place, so spends from unpublished inputs read as unspent.
    /// Resolve compressed inputs in sorted batches, retaining output order.
    fn resolve_outspends(
        &self,
        first_txout: TxOutIndex,
        output_count: usize,
        bound: Lengths,
    ) -> Result<Vec<TxOutspend>> {
        let indexer = self.indexer();
        let txin_index_reader = self.plugins().outputs.spent.spending_txin_index.reader();
        let txid_reader = indexer.vecs().transactions.txid.reader();

        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        let mut requested = Vec::new();
        let mut ordered = true;
        for index in 0..output_count {
            let Some(txin_index) = published_spend(
                txin_index_reader
                    .try_get(first_txout + Vout::from(index))
                    .data()?,
                bound,
            ) else {
                continue;
            };
            ordered &= requested
                .last()
                .is_none_or(|&(previous, _)| previous <= txin_index);
            requested.push((txin_index, index));
        }
        let mut cached_status: Option<(Height, BlockHash, Timestamp)> = None;
        let mut outspends = vec![TxOutspend::UNSPENT; output_count];
        visit_spending_positions(
            &indexer.vecs().inputs.tx_index,
            &indexer.vecs().transactions.first_txin_index,
            &mut requested,
            bound.tx_index,
            ordered,
            |index, spending_tx_index, vin| {
                let spending_txid = txid_reader.try_get(spending_tx_index).data()?;
                let spending_height: Height = tx_heights.get(spending_tx_index).data()?;
                if spending_height >= bound.height {
                    return Err(Error::UnknownTxid);
                }

                let (block_hash, block_time) = if let Some((height, hash, time)) = cached_status
                    && height == spending_height
                {
                    (hash, time)
                } else {
                    let (hash, time) = self.block_hash_and_time(spending_height)?;
                    cached_status = Some((spending_height, hash, time));
                    (hash, time)
                };

                outspends[index] = TxOutspend {
                    spent: true,
                    txid: Some(spending_txid),
                    vin: Some(vin),
                    status: Some(TxStatus::confirmed(spending_height, block_hash, block_time)),
                };
                Ok(())
            },
        )?;

        Ok(outspends)
    }
}

fn visit_spending_positions(
    input_txs: &impl ReadableVec<TxInIndex, TxIndex>,
    first_inputs: &impl ReadableVec<TxIndex, TxInIndex>,
    requested: &mut [(TxInIndex, usize)],
    tx_bound: TxIndex,
    ordered: bool,
    mut visit: impl FnMut(usize, TxIndex, Vin) -> Result<()>,
) -> Result<()> {
    if requested.is_empty() {
        return Ok(());
    }
    if ordered {
        // Already ordered reads do not revisit pages. Keep the low-allocation
        // cursor path for small transactions and sequential spending inputs.
        let mut inputs = input_txs.cursor();
        let mut firsts = first_inputs.cursor();
        for &(input, output) in requested.iter() {
            let tx = inputs.get(input.to_usize()).data()?;
            if tx < tx_bound {
                let first = firsts.get(tx.to_usize()).data()?;
                visit(output, tx, checked_vin(input, first)?)?;
            }
        }
        return Ok(());
    }
    requested.sort_unstable_by_key(|&(input, _)| input);
    let mut inputs: Vec<_> = requested
        .iter()
        .map(|&(input, _)| input.to_usize())
        .collect();
    let txs = input_txs.read_sorted_at(&inputs);
    if txs.len() != requested.len() {
        return Err(Error::Internal("Missing spending transaction index"));
    }
    // Input-to-transaction mapping is monotonic. Keep repeated transactions:
    // native sorted reads already share each decoded page across duplicates.
    let valid = txs.partition_point(|&tx| tx < tx_bound);
    inputs.truncate(valid);
    for (index, tx) in inputs.iter_mut().zip(&txs) {
        *index = tx.to_usize();
    }
    let firsts = first_inputs.read_sorted_at(&inputs);
    if firsts.len() != valid {
        return Err(Error::Internal("Missing spending first input index"));
    }
    for ((&(input, output), &tx), first) in requested[..valid].iter().zip(&txs).zip(firsts) {
        visit(output, tx, checked_vin(input, first)?)?;
    }
    Ok(())
}

fn checked_vin(input: TxInIndex, first: TxInIndex) -> Result<Vin> {
    usize::from(input)
        .checked_sub(usize::from(first))
        .filter(|vin| *vin <= usize::from(u16::MAX))
        .map(Vin::from)
        .ok_or(Error::Internal("Invalid spending input position"))
}

/// Unspent outputs have no status, and unconfirmed statuses no block.
fn confirmed_spending_block(outspend: &TxOutspend) -> Option<(BlockHash, Height)> {
    let status = outspend.status.as_ref()?;
    status.block_hash.zip(status.block_height)
}

/// What names `outspends`, serialized as `bytes` and read at `published`: once every output is
/// spent by a confirmed transaction, the newest spending block, and whether it lies below the
/// reorg window (nothing changes the answer then); until then the exact content, which a spend
/// changes at any time.
fn outspends_identity(
    outspends: &[TxOutspend],
    bytes: &[u8],
    published: Lengths,
) -> (RepresentationId, bool) {
    let content = || (RepresentationId::content(bytes), false);
    let mut newest = None;

    for outspend in outspends {
        let Some((hash, height)) = confirmed_spending_block(outspend) else {
            return content();
        };

        // Confirmed spends read under one pin: one block per height.
        if newest.is_none_or(|(_, newest_height)| height > newest_height) {
            newest = Some((hash, height));
        }
    }

    newest.map_or_else(content, |(hash, height)| {
        (RepresentationId::Block(hash), published.is_settled(height))
    })
}
