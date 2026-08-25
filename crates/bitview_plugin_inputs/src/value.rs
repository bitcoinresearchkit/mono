use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{Sats, TxInIndex, TxOutIndex};
use rayon::prelude::*;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Exit, PcoVec, ReadableVec, VecIndex, WritableVec};

const SORT_MEMORY_BUDGET: usize = 2 * 1024 * 1024 * 1024;
const BATCH_SIZE: usize = SORT_MEMORY_BUDGET / (size_of::<Entry>() + size_of::<Sats>());

pub fn compute(value: &mut PcoVec<TxInIndex, Sats>, indexer: &Indexer, exit: &Exit) -> Result<()> {
    let starting_lengths = indexer.safe_lengths();
    let txout_indexes = &indexer.vecs().inputs.txout_index;
    let dep_version = txout_indexes.version() + indexer.vecs().outputs.value.version();
    value.validate_computed_version_or_reset(dep_version)?;

    let target = txout_indexes.len();
    let starting = starting_lengths.txin_index.to_usize();
    let min = value.len().min(starting);
    if min >= target {
        return Ok(());
    }

    let value_reader = indexer.vecs().outputs.value.reader();
    let mut entries = Vec::with_capacity((target - min).min(BATCH_SIZE));
    let mut values = Vec::with_capacity((target - min).min(BATCH_SIZE));

    let mut batch_start = min;
    while batch_start < target {
        let batch_end = (batch_start + BATCH_SIZE).min(target);
        let batch_len = batch_end - batch_start;

        entries.clear();
        let mut original_index = 0_usize;
        txout_indexes.for_each_range_at(batch_start, batch_end, |txout_index| {
            entries.push(Entry {
                original_index,
                txout_index,
            });
            original_index += 1;
        });

        values.clear();
        values.resize(batch_len, Sats::MAX);
        fill_values(&mut entries, &mut values, |txout_index| {
            value_reader.get(txout_index)
        });

        value.truncate_if_needed_at(batch_start)?;
        for computed_value in values.iter().copied() {
            value.push(computed_value);
        }

        let _lock = exit.lock();
        value.write()?;

        if batch_end < target {
            info!(
                "Input values: {:.2}%",
                batch_end as f64 / target as f64 * 100.0
            );
        }
        batch_start = batch_end;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    original_index: usize,
    txout_index: TxOutIndex,
}

fn fill_values(
    entries: &mut [Entry],
    values: &mut [Sats],
    mut get_value: impl FnMut(TxOutIndex) -> Sats,
) {
    entries.par_sort_unstable_by_key(|entry| entry.txout_index);
    for entry in entries {
        if entry.txout_index.is_coinbase() {
            break;
        }
        values[entry.original_index] = get_value(entry.txout_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_read_in_txout_order_and_scattered_to_input_order() {
        let mut entries = vec![
            Entry {
                original_index: 0,
                txout_index: TxOutIndex::from(8_usize),
            },
            Entry {
                original_index: 1,
                txout_index: TxOutIndex::COINBASE,
            },
            Entry {
                original_index: 2,
                txout_index: TxOutIndex::from(2_usize),
            },
            Entry {
                original_index: 3,
                txout_index: TxOutIndex::from(5_usize),
            },
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
