use brk_error::{Error, Result};

use crate::OriginSpends;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use brk_exit::Exit;
use brk_types::{Height, Sats, TxInIndex, TxOutIndex};
use rayon::prelude::*;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, PcoVec, ReadableVec, VecIndex, WritableVec};

const SORT_MEMORY_BUDGET: usize = 2 * 1024 * 1024 * 1024;
const BATCH_SIZE: usize = SORT_MEMORY_BUDGET / (size_of::<Entry>() + size_of::<Sats>());

pub(crate) fn compute(
    value: &mut PcoVec<TxInIndex, Sats>,
    origins: &mut OriginSpends,
    indexer: &Indexer,
    mappings: &Mappings,
    from: Height,
    end: Height,
    exit: &Exit,
) -> Result<()> {
    let vecs = indexer.vecs();
    let first = &vecs.inputs.first_txin_index;
    let txout_indexes = &vecs.inputs.txout_index;
    if from > end || usize::from(end) > first.len() {
        return Err(Error::NotFound("invalid input computation range".into()));
    }
    let input_at = |h: usize| -> usize {
        first
            .collect_one(Height::from(h))
            .map_or(txout_indexes.len(), |i| i.to_usize())
    };
    value.validate_computed_version_or_reset(
        txout_indexes.version() + vecs.outputs.value.version(),
    )?;
    value.truncate_if_needed_at(input_at(usize::from(from)))?;
    origins.prepare(indexer, value, from)?;
    if value.len() < input_at(origins.len()) {
        origins.truncate(complete_height(origins.len(), value.len(), &input_at))?;
    }
    let start = origins.len();
    let end = usize::from(end);
    if start == end {
        let _lock = exit.lock();
        value.write()?;
        return Ok(());
    }
    let mut boundaries = first.collect_range_at(start, end);
    if boundaries.len() != end - start {
        return Err(Error::NotFound("incomplete input boundaries".into()));
    }
    boundaries.push(TxInIndex::from(input_at(end)));
    let stored_end = boundaries
        .partition_point(|i| i.to_usize() <= value.len())
        .saturating_sub(1);
    // An interrupted older writer may stop inside a block. Resolve that whole block again.
    value.truncate_if_needed_at(boundaries[stored_end].to_usize())?;
    let tx_heights = mappings.tx_heights.read();
    let reader = vecs.outputs.value.reader();
    debug_assert!(reader.len() < Entry::COINBASE_TXOUT_INDEX);
    // Fixed capacities preserve the existing combined 2 GiB sorting-buffer budget.
    let target = boundaries.last().unwrap().to_usize();
    let mut entries = Vec::with_capacity((target - value.len()).min(BATCH_SIZE));
    let mut values = Vec::with_capacity((target - boundaries[0].to_usize()).min(BATCH_SIZE));
    let mut offset = 0;
    while offset < boundaries.len() - 1 {
        let retained = offset < stored_end;
        let limit = if retained {
            stored_end
        } else {
            boundaries.len() - 1
        };
        let next = batch_end(&boundaries, offset, limit, BATCH_SIZE)?;
        let from = boundaries[offset].to_usize();
        let to = boundaries[next].to_usize();
        if retained {
            // Recovery/backfill only; newly resolved values never take this decoding path.
            value.collect_range_into_at(from, to, &mut values);
            if values.len() != to - from {
                return Err(Error::NotFound("incomplete retained input values".into()));
            }
        } else {
            entries.clear();
            txout_indexes.for_each_range_at(from, to, |index| {
                entries.push(Entry::new(entries.len(), index))
            });
            if entries.len() != to - from {
                return Err(Error::NotFound("incomplete input references".into()));
            }
            values.clear();
            values.resize(to - from, Sats::MAX);
            fill_values(&mut entries, &mut values, |index| reader.get(index));
            for &amount in &values {
                value.push(amount);
            }
            let _lock = exit.lock();
            value.write()?;
        }
        // Publish origins only after their individual values have been written.
        let _lock = exit.lock();
        origins.append_blocks(
            &boundaries[offset..=next],
            &values,
            &vecs.inputs.outpoint,
            &vecs.blocks.blockhash,
            &tx_heights,
        )?;
        offset = next;
        if offset < boundaries.len() - 1 {
            info!(
                "Computing input values and origins: {:.0}%",
                offset as f64 / (boundaries.len() - 1) as f64 * 100.0
            );
        }
    }
    Ok(())
}

fn complete_height(end: usize, values: usize, input_at: impl Fn(usize) -> usize) -> usize {
    let (mut low, mut high) = (0, end + 1);
    while low < high {
        let mid = low + (high - low) / 2;
        if input_at(mid) <= values {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low.saturating_sub(1)
}

fn batch_end(
    boundaries: &[TxInIndex],
    start: usize,
    limit: usize,
    capacity: usize,
) -> Result<usize> {
    let max = boundaries[start].to_usize() + capacity;
    let next = start + boundaries[start + 1..=limit].partition_point(|i| i.to_usize() <= max);
    if next == start {
        return Err(Error::NotFound("block exceeds input batch capacity".into()));
    }
    Ok(next)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Entry(u64);

impl Entry {
    const ORIGINAL_INDEX_BITS: u32 = BATCH_SIZE.next_power_of_two().ilog2();
    const ORIGINAL_INDEX_MASK: u64 = (1_u64 << Self::ORIGINAL_INDEX_BITS) - 1;
    const COINBASE_TXOUT_INDEX: usize = (u64::MAX >> Self::ORIGINAL_INDEX_BITS) as usize;

    #[inline(always)]
    fn new(original_index: usize, txout_index: TxOutIndex) -> Self {
        debug_assert!(original_index < BATCH_SIZE);
        let txout_index = if txout_index.is_coinbase() {
            Self::COINBASE_TXOUT_INDEX
        } else {
            let txout_index = txout_index.to_usize();
            debug_assert!(txout_index < Self::COINBASE_TXOUT_INDEX);
            txout_index
        };
        Self((txout_index as u64) << Self::ORIGINAL_INDEX_BITS | original_index as u64)
    }

    #[inline(always)]
    fn original_index(self) -> usize {
        (self.0 & Self::ORIGINAL_INDEX_MASK) as usize
    }

    #[inline(always)]
    fn txout_index(self) -> TxOutIndex {
        let index = (self.0 >> Self::ORIGINAL_INDEX_BITS) as usize;
        if index == Self::COINBASE_TXOUT_INDEX {
            TxOutIndex::COINBASE
        } else {
            TxOutIndex::from(index)
        }
    }
}

fn fill_values(
    entries: &mut [Entry],
    values: &mut [Sats],
    mut get_value: impl FnMut(TxOutIndex) -> Sats,
) {
    entries.par_sort_unstable();
    for &entry in entries.iter() {
        let txout_index = entry.txout_index();
        if txout_index.is_coinbase() {
            break;
        }
        values[entry.original_index()] = get_value(txout_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_end_at_block_boundaries_within_budget() {
        let boundaries = [0usize, 1, 5, 7, 13, 14].map(TxInIndex::from);
        let mut at = 0;
        let mut ranges = Vec::new();
        while at < boundaries.len() - 1 {
            let next = batch_end(&boundaries, at, boundaries.len() - 1, 6).unwrap();
            ranges.push((at, next));
            at = next;
        }
        assert_eq!(ranges, [(0, 2), (2, 3), (3, 4), (4, 5)]);
        assert!(batch_end(&boundaries, 3, 5, 5).is_err());
        assert_eq!(batch_end(&boundaries, 0, 1, 6).unwrap(), 1);
    }

    #[test]
    fn incomplete_values_rewind_to_their_block_boundary() {
        let boundaries = [0, 1, 5, 7, 13, 14];
        for (values, expected) in [(0, 0), (1, 1), (4, 1), (5, 2), (6, 2), (13, 4), (14, 5)] {
            assert_eq!(complete_height(5, values, |h| boundaries[h]), expected);
        }
    }

    #[test]
    fn values_are_read_in_txout_order_and_scattered_to_input_order() {
        let mut entries = vec![
            Entry::new(0, TxOutIndex::from(8_usize)),
            Entry::new(1, TxOutIndex::COINBASE),
            Entry::new(2, TxOutIndex::from(2_usize)),
            Entry::new(3, TxOutIndex::from(5_usize)),
        ];
        let mut values = vec![Sats::MAX; entries.len()];
        let mut reads = Vec::new();

        fill_values(&mut entries, &mut values, |txout_index| {
            reads.push(txout_index);
            Sats::from(txout_index.to_usize() * 10)
        });

        assert_eq!(
            reads,
            [
                TxOutIndex::from(2_usize),
                TxOutIndex::from(5_usize),
                TxOutIndex::from(8_usize)
            ]
        );
        assert_eq!(
            values,
            [
                Sats::from(80_usize),
                Sats::MAX,
                Sats::from(20_usize),
                Sats::from(50_usize)
            ]
        );
    }
}
