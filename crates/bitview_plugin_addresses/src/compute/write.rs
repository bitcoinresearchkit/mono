use brk_error::Result;

use std::time::Instant;

use brk_types::Height;
use rayon::prelude::*;
use tracing::info;
use vecdb::Stamp;

use crate::{
    Vecs,
    addr::{Changed, Stable},
    state::AddrStates,
};

/// Flush metrics and address state before the plugin’s scalar checkpoint.
///
/// Set `with_changes=true` near chain tip to enable rollback support. With `publish_state`, the
/// address state (rewritten in place, its changes marked by the caller) stays staged for
/// [`Vecs::publish_state`] to publish with the update's lengths; without, it is written now.
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
        let mut stable = stable.write();
        vecs.publish_state(&mut stable)?;
    }

    vecs.save_caps(addr_states, stamp, with_changes)?;

    info!("Saved distribution data in {:.2?}", i.elapsed());

    Ok(())
}

/// Address state changed in its buffers but not yet written: staged for the update's publication,
/// published from memory and waiting for its write, or left by a failed write.
pub(crate) struct UnpublishedState {
    pub stamp: Stamp,
    pub with_changes: bool,
    /// The changed addresses an update publishes from memory, until it does.
    pub changed: Option<Changed>,
}

impl Vecs {
    /// Publishes the address state the last update staged. The caller holds the state's write
    /// lock (`state.stable()`) through this and the publication of the lengths, so a reader
    /// taking the read lock before its pin sees both or neither. An update that kept its changed
    /// addresses publishes them, read before the vectors, and leaves the vectors to
    /// [`Self::write_published_state`] once its lengths are out; any other staged state is
    /// written now.
    pub fn publish_state(&mut self, stable: &mut Stable) -> Result<()> {
        let Some(unpublished) = self.unpublished_state.as_mut() else {
            return Ok(());
        };
        let (stamp, with_changes) = (unpublished.stamp, unpublished.with_changes);
        if let Some(changed) = unpublished.changed.take() {
            stable.written = Some(Height::from(stamp).incremented());
            stable.changed = Some(changed);
            return Ok(());
        }
        self.write_state(stamp, with_changes, stable)?;
        self.unpublished_state = None;
        Ok(())
    }

    /// Writes the address state [`Self::publish_state`] published from memory. Readers keep its
    /// changed addresses until the next write under the lock, so it needs no lock: no reader
    /// reads the rows it rewrites. On failure they keep reading them; the update loop stops.
    pub fn write_published_state(&mut self) -> Result<()> {
        if let Some(&UnpublishedState {
            stamp,
            with_changes,
            ..
        }) = self.unpublished_state.as_ref()
        {
            let i = Instant::now();
            // The compaction this update scheduled runs first: a header write waiting on it would
            // keep the state's readers out meanwhile.
            self.db.sync_bg_tasks()?;
            self.write_state_vecs(stamp, with_changes)?;
            self.unpublished_state = None;
            info!("Wrote address state in {:.2?}", i.elapsed());
        }
        Ok(())
    }

    /// A write that fails part way leaves the length unset until a rollback and write restore it.
    pub(crate) fn write_state(
        &mut self,
        stamp: Stamp,
        with_changes: bool,
        stable: &mut Stable,
    ) -> Result<()> {
        stable.written = None;
        stable.changed = None;
        self.write_state_vecs(stamp, with_changes)?;
        stable.written = Some(Height::from(stamp).incremented());
        Ok(())
    }

    fn write_state_vecs(&mut self, stamp: Stamp, with_changes: bool) -> Result<()> {
        self.state
            .par_iter_mut()
            .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;
        Ok(())
    }
}
