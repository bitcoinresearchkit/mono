use std::ops::Range;

use bitview_compute::write_locked;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Version;
use vecdb::{AnyVec, ReadableVec, VecIndex, WritableVec};

use crate::{
    Dependencies, TweakIndex, Vecs,
    scan::{TAPROOT_ACTIVATION, scan},
    tweak::Tweaker,
};

/// Blocks scanned between writes: about a hundred megabytes of rows at most.
const BATCH_HEIGHTS: usize = 1_000;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();
        let vecs = indexer.vecs();
        let (txs, inputs, outputs, addrs) = (
            &vecs.transactions,
            &vecs.inputs,
            &vecs.outputs,
            &vecs.addrs,
        );
        let version = txs.first_tx_index.version()
            + txs.first_txin_index.version()
            + txs.first_txout_index.version()
            + txs.position.version()
            + txs.total_size.version()
            + vecs.blocks.position.version()
            + vecs.blocks.size.version()
            + outputs.output_type.version()
            + inputs.output_type.version()
            + inputs.type_index.version()
            + inputs.outpoint.version()
            + addrs.p2pkh.bytes.version()
            + addrs.p2tr.bytes.version();

        let start = self.prepare(version, indexer.compute_start().height.to_usize(), exit)?;
        let tweaker = Tweaker::default();
        for heights in batches(start..txs.first_tx_index.len()) {
            for block in scan(indexer, &tweaker, heights)? {
                self.first_tweak_index
                    .push(TweakIndex::from(self.tx_index.len()));
                for (tx_index, tweak) in block {
                    self.tx_index.push(tx_index);
                    self.tweak.push(tweak);
                }
            }
            write_locked(self.stored_mut(), exit)?;
        }
        Ok(())
    }
}

impl Vecs {
    /// Validates every column against `version`, then keeps the blocks below `max_from` (or all
    /// stored) and their rows. Returns the first block to scan.
    fn prepare(&mut self, version: Version, max_from: usize, exit: &Exit) -> Result<usize> {
        let _lock = exit.lock();
        for vec in self.stored_mut() {
            vec.any_validate_computed_version_or_reset(version)?;
        }
        let height = max_from.min(self.first_tweak_index.len());
        let rows = self
            .first_tweak_index
            .collect_one_at(height)
            .map_or(self.tx_index.len(), usize::from);
        debug_assert_eq!(self.tx_index.len(), self.tweak.len());
        self.first_tweak_index.truncate_if_needed_at(height)?;
        self.tx_index.truncate_if_needed_at(rows)?;
        self.tweak.truncate_if_needed_at(rows)?;
        for vec in self.stored_mut() {
            vec.write()?;
        }
        Ok(height)
    }
}

/// `range` in batches ending on multiples of [`BATCH_HEIGHTS`], the blocks before Taproot's
/// activation (which hold no rows) in one.
fn batches(range: Range<usize>) -> impl Iterator<Item = Range<usize>> {
    let mut at = range.start;
    std::iter::from_fn(move || {
        (at < range.end).then(|| {
            let end = if at < TAPROOT_ACTIVATION {
                TAPROOT_ACTIVATION
            } else {
                at - at % BATCH_HEIGHTS + BATCH_HEIGHTS
            }
            .min(range.end);
            let batch = at..end;
            at = end;
            batch
        })
    })
}
