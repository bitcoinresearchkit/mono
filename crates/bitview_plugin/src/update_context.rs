use brk_exit::Exit;
use vecdb::Database;

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

    /// Schedule deferred compaction as a background task that holds the shutdown lock; the
    /// next [`crate::ComputePlugin::compute`] joins it before writing again.
    pub(crate) fn compact_database(self, db: &Database) {
        let exit = self.exit.clone();
        db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
    }
}
