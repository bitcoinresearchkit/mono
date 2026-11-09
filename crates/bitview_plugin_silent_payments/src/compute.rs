use std::ops::Range;

use bitview_compute::write_locked;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Height, TweakX, Version};
use vecdb::{AnyVec, PcoVec, PcoVecValue, ReadableVec, VecIndex, WritableVec};

use crate::{
    Dependencies, FilterByteIndex, TweakIndex, Vecs,
    scan::{TAPROOT_ACTIVATION, scan},
    tweak::Tweaker,
};

/// Blocks scanned between writes: a few hundred megabytes of rows at most.
const BATCH_HEIGHTS: usize = 1_000;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let start = context.start();
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();
        let vecs = indexer.vecs();
        let (txs, inputs, addrs) = (&vecs.transactions, &vecs.inputs, &vecs.addrs);
        let version = txs.first_tx_index.version()
            + txs.first_txin_index.version()
            + txs.position.version()
            + txs.total_size.version()
            + vecs.blocks.position.version()
            + vecs.blocks.size.version()
            + inputs.output_type.version()
            + inputs.type_index.version()
            + inputs.outpoint.version()
            + addrs.p2pkh.bytes.version()
            + addrs.p2tr.bytes.version();

        let start = self.prepare(version, start.height.to_usize(), exit)?;
        let tweaker = Tweaker::default();
        for heights in batches(start..txs.first_tx_index.len()) {
            for block in scan(indexer, &tweaker, heights)? {
                for eligible in block.eligible {
                    let [parity, x @ ..] = *eligible.tweak;
                    self.tx_index.push(eligible.tx_index);
                    self.tweak_parity.push(parity);
                    self.tweak_x.push(TweakX::from(x));
                }
                for byte in block.new_outputs_filter {
                    self.new_outputs_filter.push(byte);
                }
                for byte in block.spent_outputs_filter {
                    self.spent_outputs_filter.push(byte);
                }
                self.has_off_curve_key
                    .push(u8::from(block.has_off_curve_key));
                self.end_tweak_index
                    .push(TweakIndex::from(self.tx_index.len()));
                self.end_new_outputs_filter_index
                    .push(FilterByteIndex::from(self.new_outputs_filter.len()));
                self.end_spent_outputs_filter_index
                    .push(FilterByteIndex::from(self.spent_outputs_filter.len()));
            }
            write_locked(self.stored_mut(), exit)?;
        }
        Ok(())
    }
}

impl Vecs {
    /// Validates every column against `version`, then keeps the blocks below `max_from` that every
    /// per-block column holds (a hard stop can land between their writes), their rows and their
    /// filters' bytes. Returns the first block to scan.
    fn prepare(&mut self, version: Version, max_from: usize, exit: &Exit) -> Result<usize> {
        let _lock = exit.lock();
        for vec in self.stored_mut() {
            vec.any_validate_computed_version_or_reset(version)?;
        }
        let height = [
            self.has_off_curve_key.len(),
            self.end_tweak_index.len(),
            self.end_new_outputs_filter_index.len(),
            self.end_spent_outputs_filter_index.len(),
        ]
        .into_iter()
        .fold(max_from, usize::min);
        let rows = end_before(&self.end_tweak_index, height)?;
        let new_outputs = end_before(&self.end_new_outputs_filter_index, height)?;
        let spent_outputs = end_before(&self.end_spent_outputs_filter_index, height)?;
        self.has_off_curve_key.truncate_if_needed_at(height)?;
        self.end_tweak_index.truncate_if_needed_at(height)?;
        self.end_new_outputs_filter_index
            .truncate_if_needed_at(height)?;
        self.end_spent_outputs_filter_index
            .truncate_if_needed_at(height)?;
        self.tx_index.truncate_if_needed_at(rows)?;
        self.tweak_parity.truncate_if_needed_at(rows)?;
        self.tweak_x.truncate_if_needed_at(rows)?;
        self.new_outputs_filter.truncate_if_needed_at(new_outputs)?;
        self.spent_outputs_filter.truncate_if_needed_at(spent_outputs)?;
        for vec in self.stored_mut() {
            vec.write()?;
        }
        Ok(height)
    }
}

/// How much of a data column the blocks below `height` hold: the end of the block before it.
fn end_before<I: PcoVecValue>(ends: &PcoVec<Height, I>, height: usize) -> Result<usize>
where
    usize: From<I>,
{
    Ok(match height.checked_sub(1) {
        Some(last) => usize::from(
            ends.collect_one_at(last)
                .ok_or(Error::Internal("Silent payments block end missing"))?,
        ),
        None => 0,
    })
}

/// `range` in batches ending on multiples of [`BATCH_HEIGHTS`], the blocks before Taproot's
/// activation (which hold no rows) in one.
pub(crate) fn batches(range: Range<usize>) -> impl Iterator<Item = Range<usize>> {
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
