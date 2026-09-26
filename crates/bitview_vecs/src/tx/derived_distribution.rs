use std::{
    collections::VecDeque,
    mem,
    ops::Range,
    time::{Duration, Instant},
};

use bitview_compute::{ComputedVecValue, NumericValue, prepare_computed};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Lengths, StoredU64, TxIndex, VSize};
use schemars::JsonSchema;
use tracing::info;
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode, VecIndex, Version};

use crate::{IndexSources, PerBlockDistribution};

#[derive(Traversable)]
pub struct TxDerivedDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub block: PerBlockDistribution<T, M>,
    /// Uses the six-block window ending at the represented block.
    pub _6b: PerBlockDistribution<T, M>,
}

impl<T> TxDerivedDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let block = PerBlockDistribution::forced_import(db, name, version, indexes)?;
        let _6b = PerBlockDistribution::forced_import(db, &format!("{name}_6b"), version, indexes)?;

        Ok(Self { block, _6b })
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

    pub fn derive_from_with_skip(
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
            PerBlockDistribution::push_sorted,
        )
    }

    /// Like `derive_from_with_skip` but uses vsize-weighted percentiles for both
    /// the per-block and rolling six-block distributions.
    #[allow(clippy::too_many_arguments)]
    pub fn derive_from_with_skip_weighted(
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
            PerBlockDistribution::push_weighted_sorted,
        )
    }

    /// Read and sort each block once for both its own and its rolling distribution.
    #[allow(clippy::too_many_arguments)]
    fn compute_sorted<U: Copy + Ord>(
        &mut self,
        max_from: Height,
        first_tx_index: &impl ReadableVec<Height, TxIndex>,
        counts: &impl ReadableVec<Height, StoredU64>,
        source_name: &str,
        source_version: Version,
        skip_count: usize,
        exit: &Exit,
        mut read: impl FnMut(Range<usize>, &mut Vec<U>),
        push: impl Fn(&mut PerBlockDistribution<T>, &[U]),
    ) -> Result<()> {
        const WINDOW: usize = 6;
        let version = source_version + first_tx_index.version() + counts.version();
        let end = first_tx_index.len().min(counts.len());
        let mut outputs = [self.block.height_vecs_mut(), self._6b.height_vecs_mut()];
        let start = prepare_computed(
            outputs.as_flattened_mut(),
            version,
            usize::from(max_from).min(end),
        )?;
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
                    push(&mut self.block, &block);
                    push(&mut self._6b, &window);
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
        for target in self
            .block
            .height_vecs_mut()
            .into_iter()
            .chain(self._6b.height_vecs_mut())
        {
            target.write()?;
        }
        Ok(())
    }
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
