//! Per-block entry and transaction counts, shared by Inputs and Outputs.

use std::ops::Range;

use brk_error::{OptionData, Result};
use brk_types::{OutputType, TxIndex};
use vecdb::VecIndex;

/// Aggregated per-block counters produced by [`walk_blocks`].
pub struct BlockAggregate {
    pub entries_per_type: [u64; OutputType::COUNT],
    pub txs_per_type: [u64; OutputType::COUNT],
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
pub fn walk_blocks(
    first_transactions: &[TxIndex],
    transactions_end: usize,
    entries: Range<usize>,
    mut next_entries: impl Iterator<Item = usize>,
    coinbase: CoinbasePolicy,
    mut scan_entries: impl FnMut(usize, Option<&mut [u32; OutputType::COUNT]>),
    mut store: impl FnMut(BlockAggregate) -> Result<()>,
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
        let mut entries_per_type = [0u64; OutputType::COUNT];
        let mut txs_per_type = [0u64; OutputType::COUNT];
        for tx in start..end {
            let mut per_tx = [0u32; OutputType::COUNT];
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
