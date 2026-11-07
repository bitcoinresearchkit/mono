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
/// [`Vecs::publish_state_with`] to publish with the update's lengths; without, it is written now,
/// beside the publication readers hold to when the caller held it.
pub fn write(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    height: Height,
    with_changes: bool,
    publish_state: bool,
) -> Result<()> {
    let i = Instant::now();

    let stamp = Stamp::from(height);

    vecs.loop_vecs_mut()
        .try_for_each(|v| v.any_stamped_write_maybe_with_changes(stamp, with_changes))?;
    if !publish_state {
        vecs.write_unpublished_state()?;
    }

    vecs.save_caps(addr_states, stamp, with_changes)?;

    info!("Saved address data in {:.2?}", i.elapsed());

    Ok(())
}

/// Address state changed in its buffers but not yet written: staged for the update's publication,
/// published from memory and waiting for its write, or left by a failed write.
pub(crate) struct UnpublishedState {
    pub stamp: Stamp,
    pub with_changes: bool,
    /// The changed addresses an update publishes from memory, until it does.
    pub changed: Option<Changed>,
    /// For a write outside the update's publication: the overlay that keeps readers on the
    /// current one while it writes ([`AddrCache::held`](crate::block::AddrCache::held)), until
    /// it is in place.
    pub held: Option<Changed>,
    /// Its changed addresses are published: only the vectors' write remains.
    pub published: bool,
}

impl Vecs {
    /// Publishes the address state the last update staged with `publish_lengths`, the
    /// publication of the update's lengths, under the state's write lock: a reader taking the
    /// read lock before its pin sees both or neither.
    pub fn publish_state_with<T>(
        &mut self,
        publish_lengths: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let stable = self.state.stable();
        let mut stable = stable.write();
        self.publish_state(&mut stable)?;
        publish_lengths()
    }

    /// Publishes the address state the last update staged; the caller holds the state's write
    /// lock. An update that kept its changed addresses publishes them, read before the vectors,
    /// and leaves the vectors to [`Self::write_published_state`] once its lengths are out; any
    /// other staged state is written now.
    fn publish_state(&mut self, stable: &mut Stable) -> Result<()> {
        let Some(unpublished) = self.unpublished_state.as_mut() else {
            return Ok(());
        };
        let (stamp, with_changes) = (unpublished.stamp, unpublished.with_changes);
        if let Some(changed) = unpublished.changed.take() {
            stable.written = Some(Height::from(stamp).incremented());
            stable.changed = Some(changed);
            unpublished.published = true;
            return Ok(());
        }
        self.write_state(stamp, with_changes, stable)?;
        self.unpublished_state = None;
        Ok(())
    }

    /// Writes the address state staged outside the update's publication. With its overlay held,
    /// readers move onto it under a brief write lock and stay on the publication while the
    /// vectors are written without the lock: they read none of the rows the write rewrites,
    /// before or after a failure. Without (too large to hold, or nothing published to hold),
    /// the write excludes them.
    fn write_unpublished_state(&mut self) -> Result<()> {
        let Some(unpublished) = self.unpublished_state.as_mut() else {
            return Ok(());
        };
        let stable = self.state.stable();
        let Some(held) = unpublished.held.take() else {
            return self.publish_state(&mut stable.write());
        };
        debug_assert!(unpublished.changed.is_none(), "a held write publishes nothing");
        let (stamp, with_changes) = (unpublished.stamp, unpublished.with_changes);
        // The overlay it replaces is freed after the lock.
        let released = stable.write().changed.replace(held);
        drop(released);
        // As for a published state: a header write waiting on compaction would hold readers.
        self.db.sync_bg_tasks()?;
        self.write_state_vecs(stamp, with_changes)?;
        self.unpublished_state = None;
        Ok(())
    }

    /// Writes the address state [`Self::publish_state`] published from memory. Readers keep its
    /// changed addresses until the next write under the lock, so it needs no lock: no reader
    /// reads the rows it rewrites. On failure they keep reading them; the update loop stops.
    /// State not yet published stays staged: readers may read the rows its write rewrites.
    pub fn write_published_state(&mut self) -> Result<()> {
        match self.unpublished_state.as_ref() {
            Some(&UnpublishedState {
                stamp,
                with_changes,
                published: true,
                ..
            }) => {
                let i = Instant::now();
                // The compaction this update scheduled runs first: a header write waiting on it
                // would keep the state's readers out meanwhile.
                self.db.sync_bg_tasks()?;
                self.write_state_vecs(stamp, with_changes)?;
                self.unpublished_state = None;
                info!("Wrote address state in {:.2?}", i.elapsed());
            }
            staged => debug_assert!(
                staged.is_none(),
                "address state written before its publication"
            ),
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
