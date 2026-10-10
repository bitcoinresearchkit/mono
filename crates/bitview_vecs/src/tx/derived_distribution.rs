use std::{
    collections::VecDeque,
    mem,
    ops::Range,
    time::{Duration, Instant},
};

use bitview_collections::DistributionStats;
use bitview_compute::{ComputedVecValue, NumericValue, prepare_computed};
use bitview_primitives::{Count, Lengths};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, TxIndex, VSize, get_weighted_percentiles};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use tracing::info;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, PcoVec, ReadableVec, Rw, StorageMode, VecIndex,
    Version, WritableVec,
};

use crate::{IndexSources, PerBlock};

/// A statistic for the represented block and for the six-block window ending there.
#[derive(Clone, Traversable)]
pub struct TxWindows<A> {
    pub block: A,
    /// Uses the six-block window ending at the represented block.
    pub(crate) _6b: A,
}

impl<A> TxWindows<A> {
    fn get_mut(&mut self, six_blocks: bool) -> &mut A {
        if six_blocks {
            &mut self._6b
        } else {
            &mut self.block
        }
    }
}

/// Per-block distributions of a per-transaction value, statistic first: `{stat}.{block, _6b}`,
/// ids `{name}_{stat}` and `{name}_{stat}_6b`.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct TxDerivedDistribution<T, M: StorageMode = Rw>(
    pub DistributionStats<TxWindows<PerBlock<T, M>>>,
)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> TxDerivedDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|stat| -> Result<_> {
            Ok(TxWindows {
                block: PerBlock::import(db, &format!("{name}_{stat}"), version, indexes)?,
                _6b: PerBlock::import(db, &format!("{name}_{stat}_6b"), version, indexes)?,
            })
        })?))
    }

    /// One window's statistics.
    fn window(&mut self, six_blocks: bool) -> DistributionStats<&mut PerBlock<T>> {
        let s = &mut self.0;
        DistributionStats {
            min: s.min.get_mut(six_blocks),
            max: s.max.get_mut(six_blocks),
            pct10: s.pct10.get_mut(six_blocks),
            pct25: s.pct25.get_mut(six_blocks),
            median: s.median.get_mut(six_blocks),
            pct75: s.pct75.get_mut(six_blocks),
            pct90: s.pct90.get_mut(six_blocks),
        }
    }

    fn height_vecs_mut(&mut self) -> Vec<&mut EagerVec<PcoVec<Height, T, Budgeted>>> {
        let s = &mut self.0;
        [
            &mut s.min,
            &mut s.max,
            &mut s.median,
            &mut s.pct10,
            &mut s.pct25,
            &mut s.pct75,
            &mut s.pct90,
        ]
        .into_iter()
        .flat_map(|windows| [&mut windows.block.height, &mut windows._6b.height])
        .collect()
    }

    pub fn derive_from(
        &mut self,
        indexes: &IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        exit: &Exit,
    ) -> Result<()> {
        self.derive_from_with_skip(
            indexes,
            starting_lengths,
            first_tx_index,
            tx_index_source,
            exit,
            0,
        )
    }

    pub(crate) fn derive_from_with_skip(
        &mut self,
        indexes: &IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()> {
        let version = tx_index_source.version();
        let mut cursor = tx_index_source.cursor();
        let zero = T::from(0_usize);
        self.compute_sorted(
            starting_lengths.height,
            first_tx_index,
            &indexes.height_tx_index_count,
            tx_index_source.name(),
            version,
            skip_count,
            exit,
            |range, block| {
                cursor.advance(range.start - cursor.position());
                cursor.for_each(range.len(), |value| {
                    if skip_count == 0 || value > zero {
                        block.push(value);
                    }
                });
            },
            push_sorted,
        )
    }

    /// Like `derive_from_with_skip` but uses vsize-weighted percentiles for both
    /// the per-block and rolling six-block distributions.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn derive_from_with_skip_weighted(
        &mut self,
        indexes: &IndexSources,
        starting_lengths: &Lengths,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        tx_index_source: &impl ReadableVec<TxIndex, T>,
        vsize_source: &impl ReadableVec<TxIndex, VSize>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()> {
        let version = tx_index_source.version() + vsize_source.version();
        let mut value_cursor = tx_index_source.cursor();
        let mut vsize_cursor = vsize_source.cursor();
        let zero = T::from(0_usize);
        self.compute_sorted(
            starting_lengths.height,
            first_tx_index,
            &indexes.height_tx_index_count,
            tx_index_source.name(),
            version,
            skip_count,
            exit,
            |range, block| {
                value_cursor.advance(range.start - value_cursor.position());
                vsize_cursor.advance(range.start - vsize_cursor.position());
                vsize_cursor.for_each(range.len(), |vsize| {
                    let value = value_cursor.next().unwrap();
                    if skip_count == 0 || value > zero {
                        block.push((value, vsize));
                    }
                });
            },
            push_weighted_sorted,
        )
    }

    /// Read and sort each block once for both its own and its rolling distribution.
    #[allow(clippy::too_many_arguments)]
    fn compute_sorted<U: Copy + Ord>(
        &mut self,
        max_from: Height,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        counts: &impl ReadableVec<Height, Count>,
        source_name: &str,
        source_version: Version,
        skip_count: usize,
        exit: &Exit,
        mut read: impl FnMut(Range<usize>, &mut Vec<U>),
        push: impl Fn(DistributionStats<&mut PerBlock<T>>, &[U]),
    ) -> Result<()> {
        const WINDOW: usize = 6;
        let version = source_version + first_tx_index.version() + counts.version();
        let end = first_tx_index.len().min(counts.len());
        let mut outputs = self.height_vecs_mut();
        let start = prepare_computed(&mut outputs, version, usize::from(max_from).min(end), exit)?;
        if start < end {
            let warmup = start.saturating_sub(WINDOW - 1);
            let mut first = first_tx_index.cursor();
            let mut counts = counts.cursor();
            first.advance(warmup);
            counts.advance(warmup);
            let mut ring = VecDeque::with_capacity(WINDOW);
            let mut window = Vec::new();
            let mut buffer = Vec::new();
            let mut block = Vec::new();
            let mut last_progress = Instant::now();
            for height in warmup..end {
                let first = first.next().unwrap().to_usize();
                let count = u64::from(counts.next().unwrap()) as usize;
                let range = first + skip_count.min(count)..first + count;
                block.clear();
                block.reserve(range.len());
                read(range, &mut block);
                // Full tuples preserve exact expiry when equal rates have different weights.
                block.sort_unstable();
                let expired = (ring.len() == WINDOW).then(|| ring.pop_front().unwrap());
                update_sorted(
                    &mut window,
                    &block,
                    expired.as_deref().unwrap_or_default(),
                    &mut buffer,
                );
                if height >= start {
                    push(self.window(false), &block);
                    push(self.window(true), &window);
                }
                // Reuse the expired block's allocation for the next block.
                ring.push_back(mem::replace(&mut block, expired.unwrap_or_default()));
                if height.is_multiple_of(64) && last_progress.elapsed() >= Duration::from_secs(10) {
                    info!(
                        "Computing {source_name} distributions: block {height}/{}",
                        end - 1,
                    );
                    last_progress = Instant::now();
                }
            }
        }
        let _lock = exit.lock();
        for target in self.height_vecs_mut() {
            target.write()?;
        }
        Ok(())
    }
}

fn push_sorted<T: NumericValue + JsonSchema>(
    stats: DistributionStats<&mut PerBlock<T>>,
    values: &[T],
) {
    if let (Some(&min), Some(&max)) = (values.first(), values.last()) {
        stats.max.height.push(max);
        stats.pct90.height.push(get_percentile(values, 0.90));
        stats.pct75.height.push(get_percentile(values, 0.75));
        stats.median.height.push(get_percentile(values, 0.50));
        stats.pct25.height.push(get_percentile(values, 0.25));
        stats.pct10.height.push(get_percentile(values, 0.10));
        stats.min.height.push(min);
    } else {
        push_zeros(stats);
    }
}

fn push_weighted_sorted<T: NumericValue + JsonSchema>(
    stats: DistributionStats<&mut PerBlock<T>>,
    values: &[(T, VSize)],
) {
    if let (Some(&(min, _)), Some(&(max, _))) = (values.first(), values.last()) {
        stats.max.height.push(max);
        let [pct10, pct25, median, pct75, pct90] =
            get_weighted_percentiles(values, [0.10, 0.25, 0.50, 0.75, 0.90]);
        stats.pct90.height.push(pct90);
        stats.pct75.height.push(pct75);
        stats.median.height.push(median);
        stats.pct25.height.push(pct25);
        stats.pct10.height.push(pct10);
        stats.min.height.push(min);
    } else {
        push_zeros(stats);
    }
}

fn push_zeros<T: NumericValue + JsonSchema>(stats: DistributionStats<&mut PerBlock<T>>) {
    for vec in [
        stats.min,
        stats.max,
        stats.pct10,
        stats.pct25,
        stats.median,
        stats.pct75,
        stats.pct90,
    ] {
        vec.height.push(T::from(0_usize));
    }
}

/// A percentile of a non-empty sorted slice, nearest rank.
fn get_percentile<T: Clone>(sorted: &[T], percentile: f64) -> T {
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index].clone()
}

/// Merge a sorted block and remove the expired multiset in one pass.
fn update_sorted<T: Copy + Ord>(
    window: &mut Vec<T>,
    block: &[T],
    expired: &[T],
    buffer: &mut Vec<T>,
) {
    buffer.clear();
    buffer.reserve(window.len() + block.len() - expired.len());
    let (mut bi, mut ei) = (0, 0);
    for &value in window.iter() {
        if ei < expired.len() && value == expired[ei] {
            ei += 1;
            continue;
        }
        while bi < block.len() && block[bi] < value {
            buffer.push(block[bi]);
            bi += 1;
        }
        buffer.push(value);
    }
    debug_assert_eq!(ei, expired.len());
    buffer.extend_from_slice(&block[bi..]);
    mem::swap(window, buffer);
}
