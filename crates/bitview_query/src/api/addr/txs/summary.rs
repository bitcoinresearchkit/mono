use std::ops::Range;

use bitview_primitives::TypeIndex;
use bitview_types::{AddressTxSummary, BlockTxIndex};
use brk_types::{Address, Height, OutputType, Sats, TxIndex, Txid};
use vecdb::{ReadableVec, SparseRead, VecIndex, VecValue};

use super::ResolvedAddrChainTxs;
use crate::{Error, OptionData, Query, ReadPin, Result, api::addr::parse_addr};

impl Query {
    /// Confirmed-only summaries. Validate the cursor even for an empty address or zero limit:
    /// unknown is 422; a known pending or foreign cursor has no confirmed rows after it.
    pub fn resolve_addr_txs_summary(
        &self,
        addr: &Address,
        last_seen_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrChainTxs> {
        let bytes = parse_addr(addr)?;
        let (mempool, pin) = self.mempool_then_pin()?;
        let before = match last_seen_txid {
            Some(txid) => match self.resolve_tx_index_bounded(&txid, &pin) {
                Ok(index) => Some(index),
                Err(error) if Self::pool_answers(&error, &pin) => {
                    if let Some((mempool, view)) = self.pending_view(mempool.as_deref(), &pin)? {
                        if mempool.is_pending(&txid, &view)? {
                            return Ok(ResolvedAddrChainTxs::empty(&pin));
                        }
                        if view.in_limbo(&txid) {
                            return Err(Error::StateUpdating);
                        }
                    }
                    return Err(match error {
                        Error::UnknownTxid => Error::UnknownAfterTxid,
                        error => error,
                    });
                }
                Err(error) => return Err(error),
            },
            None => None,
        };
        if limit == 0 {
            return Ok(ResolvedAddrChainTxs::empty(&pin));
        }
        match self.find_addr_bytes(&bytes, &pin)? {
            Some((output_type, type_index)) => {
                self.resolve_addr_chain_txs_before(output_type, type_index, before, limit, &pin)
            }
            None => Ok(ResolvedAddrChainTxs::empty(&pin)),
        }
    }

    /// Read only the selected transactions' boundaries, script identities, values and status
    /// columns. The confirmed-history body reader uses this same token and anchor check.
    pub fn addr_txs_summary_resolved(
        &self,
        resolved: ResolvedAddrChainTxs,
    ) -> Result<Vec<AddressTxSummary>> {
        let pin = self.pin_safe_lengths()?;
        resolved.revalidate(self, &pin)?;
        let Some(script) = resolved.script else {
            return Ok(Vec::new());
        };
        self.script_tx_summaries(script, &resolved.txindices, &pin)
    }

    fn script_tx_summaries(
        &self,
        script: (OutputType, TypeIndex),
        txindices: &[TxIndex],
        pin: &ReadPin,
    ) -> Result<Vec<AddressTxSummary>> {
        if txindices.is_empty() {
            return Ok(Vec::new());
        }
        // Postings are unique and descending. Visit their columns ascending to reuse each
        // compressed page, then reverse the small result, without sorting or a history map.
        let indices: Vec<_> = txindices.iter().rev().map(|index| index.to_usize()).collect();
        let safe = pin.lengths();
        let vecs = self.indexer().vecs();
        let txs = &vecs.transactions;
        let mut values = vec![0_i64; indices.len()];
        {
            let ranges = transaction_ranges(&txs.first_txin_index, &indices, safe.tx_index, safe.txin_index)?;
            // Public input views map coinbase sentinels to None/zero.
            add_script_values::<_, _, true>(
                &ranges, script,
                (&vecs.inputs.output_type, vecs.inputs.type_index.view(), vecs.inputs.value.view()),
                &mut values, || self.check_deadline(),
            )?;
        }
        {
            let ranges = transaction_ranges(txs.first_txout_index.view(), &indices, safe.tx_index, safe.txout_index)?;
            add_script_values::<_, _, false>(
                &ranges, script,
                (&vecs.outputs.output_type, &vecs.outputs.type_index, &vecs.outputs.value),
                &mut values, || self.check_deadline(),
            )?;
        }
        // Owned grouped reads leave no raw reader held while another vector is read.
        let txids = txs.txid.read_sorted_at(&indices);
        if txids.len() != indices.len() {
            return Err(Error::Internal("Missing summary transaction ID"));
        }
        let heights = self.plugins().mappings.tx_heights.load();
        let mut heights = heights.cursor();
        let mut timestamps = vecs.blocks.timestamp.cursor();
        let mut first_txs = txs.first_tx_index.cursor();
        let mut cached_block = None;
        let mut rows = Vec::with_capacity(indices.len());
        for ((index, txid), value) in indices.into_iter().zip(txids).zip(values) {
            self.check_deadline()?;
            let height: Height = heights.get(TxIndex::from(index)).data()?;
            debug_assert!(height < safe.height);
            let (time, first) = match cached_block {
                Some((h, time, first)) if h == height => (time, first),
                _ => {
                    let time = timestamps.get(height.to_usize()).data()?;
                    let first = first_txs.get(height.to_usize()).data()?;
                    cached_block = Some((height, time, first));
                    (time, first)
                }
            };
            let position = index.checked_sub(first.to_usize())
                .and_then(|position| u32::try_from(position).ok())
                .ok_or(Error::Internal("Invalid summary transaction position"))?;
            rows.push(AddressTxSummary {
                txid, height, value, time, tx_position: BlockTxIndex::from(position),
            });
        }
        rows.reverse();
        Ok(rows)
    }
}

/// One grouped boundary read for any transaction-indexed start column. The final transaction
/// ends at the pin's published length, even when a writer has appended its next start.
fn transaction_ranges<I: VecIndex + VecValue>(
    firsts: &impl ReadableVec<TxIndex, I>,
    indices: &[usize],
    tx_bound: TxIndex,
    end: I,
) -> Result<Vec<Range<usize>>> {
    debug_assert!(indices.is_sorted());
    let tx_bound = tx_bound.to_usize();
    if indices.iter().any(|&index| index >= tx_bound) {
        return Err(Error::UnknownTxid);
    }
    let starts = SparseRead::try_new(firsts, indices, |index| (index + 1 < tx_bound).then_some(index + 1)).data()?;
    (0..indices.len()).map(|slot| {
        let first = starts.current(slot).to_usize();
        let next = starts.previous(slot).unwrap_or(end).to_usize();
        if first > next || next > end.to_usize() {
            return Err(Error::Internal("Summary boundaries outside published data"));
        }
        Ok(first..next)
    }).collect()
}

/// Shared exact-script contribution reader for either input or output columns. Scratch stays
/// O(rows + source chunk sizes), never O(all inputs/outputs). Only matches request value chunks.
fn add_script_values<I, T, const SPENDING: bool>(
    ranges: &[Range<usize>],
    script: (OutputType, TypeIndex),
    columns: (
        &impl ReadableVec<I, OutputType>,
        &impl ReadableVec<I, T>,
        &impl ReadableVec<I, Sats>,
    ),
    nets: &mut [i64],
    mut check: impl FnMut() -> Result<()>,
) -> Result<()>
where
    I: VecIndex,
    T: VecValue + Into<Option<TypeIndex>>,
{
    debug_assert_eq!(ranges.len(), nets.len());
    debug_assert!(ranges.windows(2).all(|pair| pair[0].end <= pair[1].start));
    let mut types = columns.0.cursor();
    let mut identities = columns.1.cursor();
    let mut values = columns.2.cursor();
    for (range, net) in ranges.iter().zip(nets) {
        // A large transaction must remain cancellable without checking a clock per input.
        for chunk in (range.start..range.end).step_by(4096) {
            check()?;
            for index in chunk..(chunk + 4096).min(range.end) {
                if types.get(index).data()? == script.0
                    && identities.get(index).data()?.into() == Some(script.1)
                {
                    // Match electrs' conversion/saturation policy; valid-money totals fit i64.
                    let value = i64::try_from(u64::from(values.get(index).data()?)).unwrap_or(0);
                    *net = if SPENDING { net.saturating_sub(value) } else { net.saturating_add(value) };
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
