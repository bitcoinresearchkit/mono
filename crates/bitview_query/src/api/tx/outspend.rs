use std::slice::from_ref;

use bitview_plugin_outputs::published_spend;
use bitview_primitives::{Lengths, TxInIndex, TxOutIndex};
use brk_mempool::ReadOnlyState;
use brk_types::{BlockHash, Height, TxIndex, TxOutspend, Txid, Vin, Vout};
use serde_json::to_vec;
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, RepresentationId, Result, read_sorted};

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
    /// Read each input chunk once in sorted order, retaining output order.
    fn resolve_outspends(
        &self,
        first_txout: TxOutIndex,
        output_count: usize,
        bound: Lengths,
    ) -> Result<Vec<TxOutspend>> {
        let indexer = self.indexer();
        let txin_index_reader = self.plugins().outputs.spent.spending_txin_index.reader();
        let mut requested = Vec::new();
        for index in 0..output_count {
            let Some(txin_index) = published_spend(
                txin_index_reader
                    .try_get(first_txout + Vout::from(index))
                    .data()?,
                bound,
            ) else {
                continue;
            };
            requested.push((txin_index.to_usize(), index));
        }
        drop(txin_index_reader);

        let mut outspends = vec![TxOutspend::UNSPENT; output_count];
        if requested.is_empty() {
            return Ok(outspends);
        }
        if !requested.is_sorted_by_key(|&(input, _)| input) {
            requested.sort_unstable_by_key(|&(input, _)| input);
        }

        // The spends' pages are scattered over years of blocks. Each column's pages are asked for
        // together before any is read, so the device serves them as one batch, not one fault at a
        // time. Round 1: each spending input's transaction, and that transaction's first input.
        let input_txs = &indexer.vecs().inputs.tx_index;
        input_txs.prefetch(requested.iter().map(|&(input, _)| TxInIndex::from(input)));
        let mut input_txs = input_txs.cursor();
        let mut first_inputs = indexer.vecs().transactions.first_txin_index.cursor();
        // (spender, its first input), in index order
        let mut spenders: Vec<(TxIndex, usize)> = Vec::new();
        // (output, its spender in `spenders`, vin)
        let mut spends = Vec::with_capacity(requested.len());
        for (input, output) in requested {
            let (start, chunk) = input_txs.chunk_at(input).data()?;
            let local = input - start;
            let spending_tx_index = chunk[local];
            if spending_tx_index >= bound.tx_index {
                break;
            }
            let first = match spenders.last() {
                Some(&(spender, first)) if spender == spending_tx_index => first,
                _ => {
                    // A transaction owns a run in the input index. Only a run that starts
                    // before this chunk needs the separate first-input column.
                    let first = if start != 0 && chunk[0] == spending_tx_index {
                        first_inputs
                            .get(spending_tx_index.to_usize())
                            .data()?
                            .to_usize()
                    } else {
                        start + chunk[..local].partition_point(|&tx| tx < spending_tx_index)
                    };
                    spenders.push((spending_tx_index, first));
                    first
                }
            };
            spends.push((output, spenders.len() - 1, checked_vin(input, first)?));
        }

        // Round 2: each spender's txid, and its block's hash and time.
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        let heights = spenders
            .iter()
            .map(|&(spender, _)| tx_heights.get(spender).data())
            .collect::<Result<Vec<Height>>>()?;
        let rows: Vec<usize> = spenders
            .iter()
            .map(|(spender, _)| spender.to_usize())
            .collect();
        let txids = read_sorted(&indexer.vecs().transactions.txid, &rows)?;
        let statuses = self.confirmed_statuses_bounded(&heights, bound)?;
        for (output, spender, vin) in spends {
            outspends[output] = TxOutspend {
                spent: true,
                txid: Some(txids[spender]),
                vin: Some(vin),
                status: Some(statuses[spender].clone()),
            };
        }
        Ok(outspends)
    }
}

fn checked_vin(input: usize, first: usize) -> Result<Vin> {
    input
        .checked_sub(first)
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
