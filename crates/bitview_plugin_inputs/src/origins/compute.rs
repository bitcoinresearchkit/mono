use bitview_plugin_indexer::Indexer;
use bitview_primitives::{SupplyState, TxInIndex};
use brk_error::{Error, Result};
use brk_types::{BlockHash, Height, Sats};
use rustc_hash::FxHashMap;
use vecdb::{AnyVec, PcoVec, ReadableVec, VecIndex};

use super::OriginSpends;

impl OriginSpends {
    pub(crate) fn prepare(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        from: Height,
    ) -> Result<()> {
        let vecs = indexer.vecs();
        self.validate_sources(
            input_values.version()
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
    pub(crate) fn append_blocks(
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
