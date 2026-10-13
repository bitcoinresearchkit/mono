use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{SupplyState, TxInIndex};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{BlockHash, Height, Sats};
use rustc_hash::FxHashMap;
use tracing::info;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::OriginSpends;

/// Inputs per batch: their values and spent outputs' heights take 384 MiB.
const BATCH_SIZE: usize = 1 << 25;

impl OriginSpends {
    /// Groups each block's spends by the height of the output they spend. Input values are the
    /// indexer's, read in order; creation heights come from the in-memory output-height map.
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &Mappings,
        from: Height,
        end: Height,
        exit: &Exit,
    ) -> Result<()> {
        let vecs = indexer.vecs();
        let first = &vecs.inputs.first_txin_index;
        let txout_indexes = &*vecs.inputs.txout_index;
        let input_values = &vecs.inputs.value;
        if from > end || usize::from(end) > first.len() {
            return Err(Error::NotFound("invalid input computation range".into()));
        }
        let input_at = |h: usize| -> usize {
            first
                .collect_one(Height::from(h))
                .map_or(txout_indexes.len(), |i| i.to_usize())
        };
        {
            let _lock = exit.lock();
            self.prepare(indexer, from)?;
        }
        let start = self.len();
        let end = usize::from(end);
        if start == end {
            return Ok(());
        }
        let mut boundaries = first.collect_range_at(start, end);
        if boundaries.len() != end - start {
            return Err(Error::NotFound("incomplete input boundaries".into()));
        }
        boundaries.push(TxInIndex::from(input_at(end)));
        let output_heights = mappings.output_heights.read();
        let target = boundaries.last().unwrap().to_usize();
        let mut values = Vec::with_capacity((target - boundaries[0].to_usize()).min(BATCH_SIZE));
        let mut heights = Vec::with_capacity(values.capacity());
        let mut offset = 0;
        while offset < boundaries.len() - 1 {
            let next = batch_end(&boundaries, offset, boundaries.len() - 1, BATCH_SIZE)?;
            let from = boundaries[offset].to_usize();
            let to = boundaries[next].to_usize();
            input_values.collect_range_into_at(from, to, &mut values);
            if values.len() != to - from {
                return Err(Error::NotFound("incomplete input values".into()));
            }
            heights.clear();
            let mut references = txout_indexes.cursor();
            references.try_for_each_range_at(from, to, |index| {
                heights.push(if index.is_coinbase() {
                    Height::ZERO
                } else {
                    output_heights
                        .get_shared(index)
                        .ok_or_else(|| Error::NotFound("spent output creation height".into()))?
                });
                Ok::<_, Error>(())
            })?;
            let _lock = exit.lock();
            self.append_blocks(
                &boundaries[offset..=next],
                &values,
                &heights,
                &vecs.blocks.blockhash,
            )?;
            offset = next;
            if offset < boundaries.len() - 1 {
                info!(
                    "Computing input origins: {:.0}%",
                    offset as f64 / (boundaries.len() - 1) as f64 * 100.0
                );
            }
        }
        Ok(())
    }

    fn prepare(&mut self, indexer: &Indexer, from: Height) -> Result<()> {
        let vecs = indexer.vecs();
        self.validate_sources(
            vecs.inputs.value.version()
                + vecs.inputs.txout_index.version()
                + vecs.inputs.first_txin_index.version()
                + vecs.outputs.first_txout_index.version(),
        )?;
        if usize::from(from) < self.len() {
            self.truncate(usize::from(from))?;
        }
        if self.len() > 0
            && Some(self.hash(self.len() - 1)?)
                != vecs
                    .blocks
                    .blockhash
                    .collect_one(Height::from(self.len() - 1))
        {
            return Err(Error::NotFound("movement chain prefix mismatch".into()));
        }
        Ok(())
    }

    /// Appends complete blocks from values and origins resolved into input order.
    fn append_blocks(
        &mut self,
        boundaries: &[TxInIndex],
        values: &[Sats],
        heights: &[Height],
        hashes: &impl ReadableVec<Height, BlockHash>,
    ) -> Result<()> {
        if boundaries.len() < 2
            || boundaries.windows(2).any(|w| w[0] >= w[1])
            || boundaries.last().unwrap().to_usize() - boundaries[0].to_usize() != values.len()
            || heights.len() != values.len()
        {
            return Err(Error::NotFound("incomplete origin block boundaries".into()));
        }
        let start = self.len();
        let end = start + boundaries.len() - 1;
        let hashes = hashes.collect_range_at(start, end);
        if hashes.len() != end - start {
            return Err(Error::NotFound("incomplete origin block hashes".into()));
        }
        let base = boundaries[0].to_usize();
        let mut spent = FxHashMap::default();
        for (offset, pair) in boundaries.windows(2).enumerate() {
            // The first input belongs to the coinbase and has no previous output.
            let from = pair[0].to_usize() + 1;
            let to = pair[1].to_usize();
            spent.clear();
            for (&value, &origin) in values[from - base..to - base]
                .iter()
                .zip(&heights[from - base..to - base])
            {
                *spent.entry(origin).or_default() += SupplyState {
                    value,
                    utxo_count: 1,
                };
            }
            let h = start + offset;
            self.push(Height::from(h), hashes[offset], &spent)?;
            if (h + 1).is_multiple_of(10_000) || h + 1 == end {
                self.commit()?;
            }
        }
        Ok(())
    }
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
