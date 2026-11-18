use std::time::Duration;

use brk_exit::Exit;
use vecdb::Database;

/// How long compaction waits after an update, off its save path: the next block often lands
/// first and reuses the freed space.
const COMPACTION_DELAY: Duration = Duration::from_secs(5);

/// Shared control state for one complete plugin-composition update.
#[derive(Clone, Copy)]
pub struct UpdateContext<'a> {
    exit: &'a Exit,
}

impl<'a> UpdateContext<'a> {
    pub const fn new(exit: &'a Exit) -> Self {
        Self { exit }
    }

    pub const fn exit(self) -> &'a Exit {
        self.exit
    }

    /// Schedules deferred compaction as a background task, unless the database has nothing to
    /// reclaim: a wait, then compaction under the shutdown lock, so a quit during the wait never
    /// waits for it. A join during the wait (the next [`crate::ComputePlugin::compute`] before
    /// it writes again, or a drop) cancels it, so it never runs on an update's path: the space
    /// stays marked to reclaim, and that compute schedules it again once it succeeds. A plugin
    /// computed outside it (a consumer of the shared walk) calls this itself once its
    /// computation succeeds.
    pub fn compact_database(self, db: &Database) {
        if !db.needs_compaction() {
            return;
        }
        let exit = self.exit.clone();
        db.run_bg(move |db| {
            if !db.bg_sleep(COMPACTION_DELAY) {
                return Ok(());
            }
            let _lock = exit.lock();
            db.compact()
        });
    }
}
