use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{Vecs, state::UTXOStates};

/// Writes the cohorts' outputs, then the plugin's scalar checkpoint at `height`: the outputs
/// resume by length and carry no stamp, the caps keep theirs and, `with_changes`, their undo.
pub fn write(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    height: Height,
    with_changes: bool,
) -> Result<()> {
    let i = Instant::now();

    vecs.cohorts
        .par_iter_vecs_mut()
        .try_for_each(|v| v.write().map(drop))?;

    vecs.save_caps(utxo_states, Stamp::from(height), with_changes)?;

    info!("Saved UTXO data in {:.2?}", i.elapsed());

    Ok(())
}
