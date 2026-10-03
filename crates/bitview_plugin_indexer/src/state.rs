use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use bitview_plugin::Publication;
use brk_types::Lengths;
use parking_lot::RwLock;

use crate::SafeLengths;

/// One writer publishes complete bounds; only rollback drains pinned readers.
pub struct State {
    pub publication: Publication,
    lengths: ArcSwap<Lengths>,
    reorg: Arc<RwLock<()>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            publication: Default::default(),
            lengths: ArcSwap::from_pointee(Lengths::default()),
            reorg: Arc::new(RwLock::new(())),
        }
    }

    pub fn lengths(&self) -> Lengths {
        **self.lengths.load()
    }

    pub fn pin_for(&self, timeout: Duration) -> Option<SafeLengths> {
        self.reorg
            .try_read_arc_for(timeout)
            .map(|guard| SafeLengths::new(guard, self.lengths()))
    }

    pub fn try_pin(&self) -> Option<SafeLengths> {
        self.reorg
            .try_read_arc()
            .map(|guard| SafeLengths::new(guard, self.lengths()))
    }

    pub fn pin_recursive(&self) -> SafeLengths {
        SafeLengths::new(self.reorg.read_arc_recursive(), self.lengths())
    }

    pub fn finish_update(&self, next: Lengths) {
        let current = self.lengths();
        if current == next {
            return;
        }
        debug_assert!(
            {
                let mut clamped = next;
                clamped.clamp_to(&current);
                clamped == current
            },
            "length regression"
        );
        self.lengths.store(Arc::new(next));
    }

    pub fn lower_before(&self, starting: &Lengths) {
        let current = self.lengths();
        let mut lowered = current;
        lowered.clamp_to(starting);
        if lowered == current {
            return;
        }
        // Acquire exclusion before publishing the smaller prefix. Once old
        // readers drain, new readers can use the unaffected prefix during undo.
        let guard = self.reorg.write();
        self.lengths.store(Arc::new(lowered));
        drop(guard);
    }
}
