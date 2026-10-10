use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{Vecs, state::AddrStates};

/// Flush metrics and address state before the plugin’s scalar checkpoint.
///
/// Set `with_changes=true` near chain tip to enable rollback support.
pub fn write(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    height: Height,
    with_changes: bool,
) -> Result<()> {
    info!("Saving addresses data...");

    let i = Instant::now();

    let stamp = Stamp::from(height);

    vecs.stateful_vecs_mut()
        .into_par_iter()
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;

    vecs.save_caps(addr_states, stamp, with_changes)?;

    info!("Saved distribution data in {:.2?}", i.elapsed());

    Ok(())
}
