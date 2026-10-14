use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{Vecs, state::AddrStates};

/// Flush metrics and address state before the plugin’s scalar checkpoint.
///
/// Set `with_changes=true` near chain tip to enable rollback support. With `publish_state`, the
/// address state (rewritten in place, its changes marked by the caller) stays staged until
/// [`Vecs::publish_state`] writes it with the update's publication.
pub fn write(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    height: Height,
    with_changes: bool,
    publish_state: bool,
) -> Result<()> {
    info!("Saving addresses data...");

    let i = Instant::now();

    let stamp = Stamp::from(height);

    vecs.loop_vecs_mut()
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;
    if !publish_state {
        let stable = vecs.state.stable();
        let mut written = stable.write();
        vecs.publish_state(&mut written)?;
    }

    vecs.save_caps(addr_states, stamp, with_changes)?;

    info!("Saved distribution data in {:.2?}", i.elapsed());

    Ok(())
}

impl Vecs {
    /// Writes the address state the last update staged. The caller holds the state's write lock
    /// (`state.stable()`, whose length is `written`) through this and the publication of the
    /// lengths, so a reader taking the read lock before its pin sees both or neither.
    pub fn publish_state(&mut self, written: &mut Option<Height>) -> Result<()> {
        if let Some((stamp, with_changes)) = self.unpublished_state {
            self.write_state(stamp, with_changes, written)?;
            self.unpublished_state = None;
        }
        Ok(())
    }

    /// A write that fails part way leaves `written` unset until a rollback and write restore it.
    pub(crate) fn write_state(
        &mut self,
        stamp: Stamp,
        with_changes: bool,
        written: &mut Option<Height>,
    ) -> Result<()> {
        *written = None;
        self.state
            .par_iter_mut()
            .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;
        *written = Some(Height::from(stamp).incremented());
        Ok(())
    }
}
