use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{
    Vecs,
    state::{AddrStates, UTXOStates},
};

/// Flush metrics and address state before the shared scalar checkpoint.
///
/// Set `with_changes=true` near chain tip to enable rollback support.
pub fn write(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    addr_states: &mut AddrStates,
    height: Height,
    with_changes: bool,
) -> Result<()> {
    info!("Saving distribution data...");

    let i = Instant::now();

    let stamp = Stamp::from(height);

    vecs.addr_state
        .par_iter_mut()
        .chain(vecs.addrs.par_iter_stateful_height_mut())
        .chain(vecs.cohorts.par_iter_vecs_mut())
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;

    vecs.save_caps(utxo_states, addr_states, stamp, with_changes)?;

    info!("Saved distribution data in {:.2?}", i.elapsed());

    Ok(())
}
