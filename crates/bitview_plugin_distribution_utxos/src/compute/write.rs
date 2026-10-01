use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{Vecs, state::UTXOStates};

/// Flush UTXO metrics before this plugin’s scalar checkpoint.
///
/// Set `with_changes=true` near chain tip to enable rollback support.
pub fn write(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    height: Height,
    with_changes: bool,
) -> Result<()> {
    info!("Saving distribution data...");

    let i = Instant::now();

    let stamp = Stamp::from(height);

    vecs.cohorts
        .par_iter_vecs_mut()
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;

    vecs.save_caps(utxo_states, stamp, with_changes)?;

    info!("Saved distribution data in {:.2?}", i.elapsed());

    Ok(())
}
