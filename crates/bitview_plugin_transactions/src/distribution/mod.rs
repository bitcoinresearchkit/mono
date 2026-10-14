mod per_tx;

pub use per_tx::PerTxDistribution;

use std::{
    ops::Range,
    time::{Duration, Instant},
};

use bitview_compute::{
    ComputedVecValue, DistributionStats, NumericValue, prepare_computed, write_locked,
};
use bitview_primitives::{Count, Lengths};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, PerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, TxIndex, VSize, get_weighted_percentiles};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use tracing::info;
use vecdb::{Database, ReadableVec, Rw, StorageMode, VecIndex, Version, WritableVec};

/// Share of a block's vsize at each end whose extreme rates bound the effective fee rate range.
const EDGE_SHARE: f64 = 0.02;

/// Per-block distributions of a per-transaction value, statistic first, ids `{name}_{stat}`.
#[derive(Deref, DerefMut, Traversable)]
pub struct TxDistribution<T, M: StorageMode = Rw>(pub DistributionStats<PerBlock<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> TxDistribution<T>
where
    T: NumericValue + JsonSchema + Ord,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|stat| {
            PerBlock::import(db, &format!("{name}_{stat}"), version, indexes)
        })?))
    }

    /// Each block's values counted equally, as mempool.space's fee percentiles: sorted, min the
    /// first, max the last and percentile `p` at `sorted[max(0, floor(n * p) - 1)]`. The block's
    /// first `skip` transactions (its coinbase) are left out.
    pub fn compute(
        &mut self,
        indexes: &IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        source: &impl ReadableVec<TxIndex, T>,
        skip: usize,
        exit: &Exit,
    ) -> Result<()> {
        let mut cursor = source.cursor();
        self.compute_blocks(
            starting_lengths.height,
            first_tx_index,
            &indexes.height_tx_index_count,
            source.name(),
            source.version(),
            skip,
            exit,
            |range, block| {
                cursor.advance(range.start - cursor.position());
                cursor.for_each(range.len(), |value| block.push(value));
            },
            push_counted,
        )
    }

    /// Each block's non-coinbase rates weighted by vsize, with Bitcoin Core's `getblockstats`
    /// rank rule: the first rate whose cumulative vsize reaches `p` of the total. Min and max
    /// follow mempool.space's effective fee heuristic: the lower of the 1st percentile and the
    /// lowest rate among the transactions overlapping the block's last 2% of vsize (block
    /// order), the higher of the 99th percentile and the highest among those overlapping its
    /// first 2%.
    pub fn compute_weighted(
        &mut self,
        indexes: &IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        source: &impl ReadableVec<TxIndex, T>,
        vsizes: &impl ReadableVec<TxIndex, VSize>,
        exit: &Exit,
    ) -> Result<()> {
        let mut value_cursor = source.cursor();
        let mut vsize_cursor = vsizes.cursor();
        self.compute_blocks(
            starting_lengths.height,
            first_tx_index,
            &indexes.height_tx_index_count,
            source.name(),
            source.version() + vsizes.version(),
            1,
            exit,
            |range, block| {
                value_cursor.advance(range.start - value_cursor.position());
                vsize_cursor.advance(range.start - vsize_cursor.position());
                vsize_cursor.for_each(range.len(), |vsize| {
                    block.push((value_cursor.next().unwrap(), vsize));
                });
            },
            push_weighted,
        )
    }

    /// Reads each block's values (`read`, block order) and writes their statistics (`push`).
    #[allow(clippy::too_many_arguments)]
    fn compute_blocks<U>(
        &mut self,
        max_from: Height,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        counts: &impl ReadableVec<Height, Count>,
        source_name: &str,
        source_version: Version,
        skip: usize,
        exit: &Exit,
        mut read: impl FnMut(Range<usize>, &mut Vec<U>),
        push: impl Fn(&mut DistributionStats<PerBlock<T>>, &mut [U]),
    ) -> Result<()> {
        let version = source_version + first_tx_index.version() + counts.version() + Version::TWO;
        let end = first_tx_index.len().min(counts.len());
        let start = prepare_computed(
            self.iter_any_stored_mut().collect::<Vec<_>>(),
            version,
            usize::from(max_from).min(end),
            exit,
        )?;
        let mut first = first_tx_index.cursor();
        let mut counts = counts.cursor();
        first.advance(start);
        counts.advance(start);
        let mut block = Vec::new();
        let mut last_progress = Instant::now();
        for height in start..end {
            let first = first.next().unwrap().to_usize();
            let count = u64::from(counts.next().unwrap()) as usize;
            block.clear();
            read(first + skip.min(count)..first + count, &mut block);
            push(&mut self.0, &mut block);
            if height.is_multiple_of(64) && last_progress.elapsed() >= Duration::from_secs(10) {
                info!(
                    "Computing {source_name} distributions: block {height}/{}",
                    end - 1
                );
                last_progress = Instant::now();
            }
        }
        write_locked(self.iter_any_stored_mut(), exit)
    }
}

fn push_counted<T: NumericValue + JsonSchema + Ord>(
    stats: &mut DistributionStats<PerBlock<T>>,
    values: &mut [T],
) {
    values.sort_unstable();
    let (Some(&min), Some(&max)) = (values.first(), values.last()) else {
        return push_undefined(stats);
    };
    let at = |p: f64| values[((values.len() as f64 * p) as usize).saturating_sub(1)];
    stats.min.height.push(min);
    stats.pct10.height.push(at(0.10));
    stats.pct25.height.push(at(0.25));
    stats.median.height.push(at(0.50));
    stats.pct75.height.push(at(0.75));
    stats.pct90.height.push(at(0.90));
    stats.max.height.push(max);
}

fn push_weighted<T: NumericValue + JsonSchema + Ord>(
    stats: &mut DistributionStats<PerBlock<T>>,
    block: &mut [(T, VSize)],
) {
    if block.is_empty() {
        return push_undefined(stats);
    }
    let total: u64 = block.iter().map(|&(_, vsize)| u64::from(vsize)).sum();
    let edge = total as f64 * EDGE_SHARE;
    let head_max = edge_rate(block.iter().copied(), edge, Ord::max);
    let tail_min = edge_rate(block.iter().rev().copied(), edge, Ord::min);
    block.sort_unstable();
    let [pct1, pct10, pct25, median, pct75, pct90, pct99] =
        get_weighted_percentiles(block, [1, 10, 25, 50, 75, 90, 99]);
    stats.min.height.push(pct1.min(tail_min));
    stats.pct10.height.push(pct10);
    stats.pct25.height.push(pct25);
    stats.median.height.push(median);
    stats.pct75.height.push(pct75);
    stats.pct90.height.push(pct90);
    stats.max.height.push(pct99.max(head_max));
}

/// The extreme rate, by `pick`, of the transactions overlapping the first `edge` vbytes of `txs`
/// (at least the first one).
fn edge_rate<T: Copy>(
    mut txs: impl Iterator<Item = (T, VSize)>,
    edge: f64,
    pick: impl Fn(T, T) -> T,
) -> T {
    let (mut extreme, vsize) = txs.next().unwrap();
    let mut before = u64::from(vsize);
    for (rate, vsize) in txs {
        if before as f64 >= edge {
            break;
        }
        extreme = pick(extreme, rate);
        before += u64::from(vsize);
    }
    extreme
}

/// No values: the type's undefined value, null for rates and zero for amounts.
fn push_undefined<T: NumericValue + JsonSchema>(stats: &mut DistributionStats<PerBlock<T>>) {
    for stat in stats.as_mut_array() {
        stat.height.push(T::from(f64::NAN));
    }
}
