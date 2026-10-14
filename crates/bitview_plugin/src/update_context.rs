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

    /// Schedules deferred compaction as a background task: a cancellable wait, then compaction
    /// under the shutdown lock, so a quit during the wait never waits for it. The next
    /// [`crate::ComputePlugin::compute`] joins it before writing again.
    pub(crate) fn compact_database(self, db: &Database) {
        let exit = self.exit.clone();
        db.run_bg(move |db| {
            db.bg_sleep(COMPACTION_DELAY);
            let _lock = exit.lock();
            db.compact()
        });
    }
}
