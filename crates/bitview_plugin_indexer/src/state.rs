use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use arc_swap::ArcSwap;
use bitview_primitives::Lengths;
use brk_types::BlockHash;

use crate::SafeLengths;

/// What readers see: the indexed lengths every plugin has computed to, the tip block they end
/// with, and how many publications came before. Rows below these lengths only change in a
/// rollback.
pub(crate) struct Published {
    pub(crate) lengths: Lengths,
    pub(crate) revision: u64,
    pub(crate) tip: BlockHash,
    /// The tip of the publication this one followed; the tip itself after a rollback.
    pub(crate) parent: BlockHash,
    /// Shared by every publication since the last rollback: a rollback waits until no reader
    /// holds one of the publications before it.
    era: Arc<()>,
}

/// One writer publishes complete bounds; pins never wait, and only a rollback waits for the pins
/// of the publications before it.
pub struct State {
    published: ArcSwap<Published>,
    /// Serializes lowerings: plugins computing side by side may each lower the publication.
    lowering: Mutex<()>,
}

impl State {
    pub fn new() -> Self {
        Self {
            published: ArcSwap::from_pointee(Published {
                lengths: Lengths::default(),
                revision: 0,
                tip: BlockHash::default(),
                parent: BlockHash::default(),
                era: Arc::default(),
            }),
            lowering: Mutex::default(),
        }
    }

    pub fn lengths(&self) -> Lengths {
        self.published.load().lengths
    }

    pub fn pin(&self) -> SafeLengths {
        SafeLengths::new(self.published.load_full())
    }

    pub fn finish_update(&self, next: Lengths, tip: BlockHash) {
        let current = self.published.load_full();
        debug_assert!(
            {
                let mut clamped = next;
                clamped.clamp_to(&current.lengths);
                clamped == current.lengths
            },
            "length regression"
        );
        self.published.store(Arc::new(Published {
            lengths: next,
            revision: current.revision + 1,
            tip,
            parent: current.tip,
            era: Arc::clone(&current.era),
        }));
    }

    /// Publishes the prefix below `starting` (ending with block `tip`), then waits until no
    /// reader holds an earlier publication, so the caller may change rows past it.
    pub fn lower_before(&self, starting: &Lengths, tip: BlockHash) {
        let _lowering = self
            .lowering
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let current = self.published.load_full();
        let mut lowered = current.lengths;
        lowered.clamp_to(starting);
        if lowered == current.lengths {
            return;
        }
        let era = Arc::clone(&current.era);
        let revision = current.revision + 1;
        drop(current);
        self.published.store(Arc::new(Published {
            lengths: lowered,
            revision,
            tip,
            parent: tip,
            era: Arc::default(),
        }));
        // New readers pin the lowered prefix; the earlier ones finish their reads.
        while Arc::strong_count(&era) > 1 {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
