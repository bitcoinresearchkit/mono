use std::time::Duration;

use bitview_primitives::Lengths;
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

/// What a dependent plugin computes under: the update's control state and its recompute point,
/// fixed once the indexer's pass has run.
#[derive(Clone, Copy)]
pub struct DependentContext<'a> {
    update: UpdateContext<'a>,
    start: Lengths,
}

impl<'a> DependentContext<'a> {
    pub const fn start(self) -> Lengths {
        self.start
    }

    pub const fn exit(self) -> &'a Exit {
        self.update.exit
    }
}

impl<'a> From<DependentContext<'a>> for UpdateContext<'a> {
    fn from(context: DependentContext<'a>) -> Self {
        context.update
    }
}

impl<'a> UpdateContext<'a> {
    pub const fn new(exit: &'a Exit) -> Self {
        Self { exit }
    }

    /// The dependents' context, recomputing from `start`.
    pub const fn dependents(self, start: Lengths) -> DependentContext<'a> {
        DependentContext {
            update: self,
            start,
        }
    }

    pub const fn exit(self) -> &'a Exit {
        self.exit
    }

    /// Schedules deferred compaction as a background task, unless the database has nothing to
    /// reclaim: a cancellable wait, then compaction under the shutdown lock, so a quit during the
    /// wait never waits for it. The next [`crate::ComputePlugin::compute`] joins it before
    /// writing again.
    pub(crate) fn compact_database(self, db: &Database) {
        if !db.needs_compaction() {
            return;
        }
        let exit = self.exit.clone();
        db.run_bg(move |db| {
            db.bg_sleep(COMPACTION_DELAY);
            let _lock = exit.lock();
            db.compact()
        });
    }
}
