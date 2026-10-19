use std::ops::Range;

use bitview_compute::prepare_computed;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Lengths;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, TxIndex, Version};
use vecdb::{AnyStoredVec, ReadableVec, VecIndex};

/// Resumes a pass over whole blocks that writes per-transaction (`txs`) and per-block
/// (`blocks`) outputs at the first block either group lacks, bounded by the published
/// `starting` lengths. Validates every output against `version`, then truncates both groups to
/// that block and persists the truncation, even with nothing left to compute.
///
/// Returns the blocks left up to `end_height` and the first one's first transaction (`end_tx`,
/// the transactions the pass can read, when none is left).
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_blocks(
    txs: &mut [&mut dyn AnyStoredVec],
    blocks: &mut [&mut dyn AnyStoredVec],
    version: Version,
    starting: Lengths,
    end_tx: usize,
    end_height: usize,
    mappings: &MappingsVecs,
    first_tx: &impl ReadableVec<Height, TxIndex>,
    exit: &Exit,
) -> Result<(Range<usize>, usize)> {
    let tx_len = prepare_computed(&mut *txs, version, starting.tx_index.to_usize(), exit)?;
    let block_len = prepare_computed(&mut *blocks, version, starting.height.to_usize(), exit)?;
    let start = block_len.min(
        mappings
            .tx_heights
            .resume_height(tx_len, end_tx, end_height),
    );
    let start_tx = first_tx
        .collect_one_at(start)
        .map_or(end_tx, |tx| tx.to_usize());
    let _lock = exit.lock();
    for vec in txs.iter_mut() {
        vec.any_truncate_if_needed_at(start_tx)?;
        vec.write()?;
    }
    for vec in blocks.iter_mut() {
        vec.any_truncate_if_needed_at(start)?;
        vec.write()?;
    }
    Ok((start..end_height, start_tx))
}
