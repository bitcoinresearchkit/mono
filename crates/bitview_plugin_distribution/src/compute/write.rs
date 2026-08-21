use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Stamp, VecIndex, WritableVec};

use crate::{
    Vecs,
    state::{AddrStates, BlockState, UTXOStates},
};

/// Flush checkpoint to disk (pure I/O, no processing).
///
/// Writes all accumulated data in parallel:
/// - Cohort stored vectors (parallel internally)
/// - Height-indexed vectors
/// - Address indexes and data
/// - Transaction output index mappings
/// - Chain state
///
/// Set `with_changes=true` near chain tip to enable rollback support.
pub fn write(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    addr_states: &mut AddrStates,
    height: Height,
    chain_state: &[BlockState],
    min_supply_modified: Option<Height>,
    with_changes: bool,
) -> Result<()> {
    info!("Writing to disk...");

    let i = Instant::now();

    let stamp = Stamp::from(height);

    // Incremental supply_state write: only rewrite from the earliest modified height
    let supply_state_len = vecs.supply_state.len();
    let truncate_to =
        min_supply_modified.map_or(supply_state_len, |h| h.to_usize().min(supply_state_len));
    vecs.supply_state
        .truncate_if_needed(Height::from(truncate_to))?;
    for block_state in &chain_state[truncate_to..] {
        vecs.supply_state.push(block_state.supply);
    }

    vecs.addr_state
        .par_iter_mut()
        .chain(vecs.addrs.par_iter_stateful_height_mut())
        .chain(
            [
                &mut vecs.supply_state as &mut dyn AnyStoredVec,
                vecs.coindays_created.stored_mut(),
                vecs.coinblocks_destroyed.stored_mut(),
            ]
            .into_par_iter(),
        )
        .chain(vecs.cohorts.par_iter_vecs_mut())
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;

    // Commit states after vec writes
    let cleanup = with_changes;
    utxo_states.write(height, cleanup)?;
    addr_states.write(height, cleanup)?;

    info!("Wrote in {:?}", i.elapsed());

    Ok(())
}
