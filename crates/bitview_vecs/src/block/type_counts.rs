use std::ops::Range;

use bitview_compute::prepare_computed;
use bitview_primitives::Count;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, TxIndex, Version};
use vecdb::{AnyStoredVec, ReadableVec, VecIndex, WritableVec};

use crate::CachedSeries;

const WRITE_INTERVAL: usize = 10_000;

/// A member's position among the `N` counters, and its cumulative entry and transaction counts.
type TypeCountTarget<'a> = (
    usize,
    &'a mut CachedSeries<Height, Count>,
    &'a mut CachedSeries<Height, Count>,
);

/// Compute cumulative entry and transaction counts for the selected output
/// types from the indexer's transaction layout. Inputs skip coinbase
/// transactions; outputs include them.
///
/// `open_starts(first_tx)` yields each following transaction's first entry
/// index, starting with `first_tx`'s; `open_types(first_entry)` returns the
/// per-transaction counter, one slot per member (`None` skips the entries).
/// A transaction counts once per member, whatever the member's entries.
#[allow(clippy::too_many_arguments)]
pub fn compute_type_counts<'a, S, F, const N: usize>(
    targets: impl IntoIterator<Item = TypeCountTarget<'a>>,
    first_tx_index: &impl ReadableVec<Height, TxIndex>,
    txid_len: usize,
    entries_len: usize,
    max_from: Height,
    version: Version,
    coinbase: CoinbasePolicy,
    open_starts: impl FnOnce(usize) -> S,
    open_types: impl FnOnce(usize) -> F,
    exit: &Exit,
) -> Result<()>
where
    S: Iterator<Item = usize>,
    F: FnMut(usize, Option<&mut [u32; N]>),
{
    let end = first_tx_index.len();
    let mut targets: Vec<_> = targets.into_iter().collect();
    let skip = prepare_computed(
        targets
            .iter_mut()
            .flat_map(|(_, entries, txs)| {
                [
                    &mut **entries as &mut dyn AnyStoredVec,
                    &mut **txs as &mut dyn AnyStoredVec,
                ]
            })
            .collect::<Vec<_>>(),
        version,
        usize::from(max_from).min(end),
        exit,
    )?;
    if skip >= end || targets.is_empty() {
        return Ok(());
    }
    let mut entry_totals = [Count::ZERO; N];
    let mut tx_totals = [Count::ZERO; N];
    for (slot, entries, txs) in &mut targets {
        entry_totals[*slot] = entries.collect_last().unwrap_or_default();
        tx_totals[*slot] = txs.collect_last().unwrap_or_default();
    }

    let mut height = skip;
    let write = |targets: &mut [TypeCountTarget<'_>]| -> Result<()> {
        let _lock = exit.lock();
        for (_, entries, txs) in targets {
            entries.write()?;
            txs.write()?;
        }
        Ok(())
    };
    let first_transactions = first_tx_index.collect_range_at(skip, end);
    let first_tx = first_transactions
        .first()
        .expect("block range is nonempty")
        .to_usize();
    let mut starts = open_starts(first_tx);
    let first_entry = starts.next().data()?;
    walk_blocks(
        &first_transactions,
        txid_len,
        first_entry..entries_len,
        starts,
        coinbase,
        open_types(first_entry),
        |aggregate| {
            for (slot, entries, txs) in &mut targets {
                let slot = *slot;
                entry_totals[slot] += Count::from(aggregate.entries_per_type[slot]);
                tx_totals[slot] += Count::from(aggregate.txs_per_type[slot]);
                entries.push(entry_totals[slot]);
                txs.push(tx_totals[slot]);
            }
            height += 1;
            if height.is_multiple_of(WRITE_INTERVAL) {
                write(&mut targets)?;
            }
            Ok(())
        },
    )?;
    write(&mut targets)
}

/// Aggregated per-block counters produced by [`walk_blocks`].
struct BlockAggregate<const N: usize> {
    entries_per_type: [u64; N],
    txs_per_type: [u64; N],
}

/// Whether to include the first transaction in each block.
#[derive(Clone, Copy)]
pub enum CoinbasePolicy {
    Include,
    Skip,
}

/// Scan contiguous transaction entries in block order.
///
/// `next_entries` yields the remaining transaction starts after `entries.start`;
/// `entries.end` closes the last transaction. `scan_entries` consumes each
/// transaction's entries, counting by type when given a target and skipping
/// coinbase entries when given `None`. The caller owns the source cursors.
#[inline]
fn walk_blocks<const N: usize>(
    first_transactions: &[TxIndex],
    transactions_end: usize,
    entries: Range<usize>,
    mut next_entries: impl Iterator<Item = usize>,
    coinbase: CoinbasePolicy,
    mut scan_entries: impl FnMut(usize, Option<&mut [u32; N]>),
    mut store: impl FnMut(BlockAggregate<N>) -> Result<()>,
) -> Result<()> {
    let mut previous = entries.start;
    let mut next_count = |tx: usize| -> Result<usize> {
        let end = if tx + 1 < transactions_end {
            next_entries.next().data()?
        } else {
            entries.end
        };
        let count = end - previous;
        previous = end;
        Ok(count)
    };
    for (offset, first_tx) in first_transactions.iter().enumerate() {
        let first = first_tx.to_usize();
        let end = first_transactions
            .get(offset + 1)
            .map(|v| v.to_usize())
            .unwrap_or(transactions_end);
        let start = match coinbase {
            CoinbasePolicy::Include => first,
            CoinbasePolicy::Skip => {
                scan_entries(next_count(first)?, None);
                first + 1
            }
        };
        let mut entries_per_type = [0u64; N];
        let mut txs_per_type = [0u64; N];
        for tx in start..end {
            let mut per_tx = [0u32; N];
            scan_entries(next_count(tx)?, Some(&mut per_tx));
            for (i, &n) in per_tx.iter().enumerate() {
                if n > 0 {
                    entries_per_type[i] += u64::from(n);
                    txs_per_type[i] += 1;
                }
            }
        }
        store(BlockAggregate {
            entries_per_type,
            txs_per_type,
        })?;
    }
    Ok(())
}
