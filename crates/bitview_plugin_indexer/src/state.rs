use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use arc_swap::ArcSwap;
use bitview_primitives::Lengths;
use brk_types::BlockHash;

use crate::{SafeLengths, stores::AddrUnspentViews};

/// What readers see: the indexed lengths every plugin has computed to, the tip block they end
/// with, and how many publications came before. Rows below these lengths only change in a
/// rollback.
#[derive(Default)]
pub(crate) struct Published {
    pub(crate) lengths: Lengths,
    pub(crate) revision: u64,
    pub(crate) tip: BlockHash,
    /// The address-unspent stores as of `lengths`: `None` exactly while the publication is
    /// lowered, the stores holding rows past it, which spends there may have deleted.
    pub(crate) unspent: Option<AddrUnspentViews>,
    /// Shared by every publication since the last rollback: a rollback waits until no reader
    /// holds one of the publications before it.
    era: Arc<()>,
}

/// One writer publishes complete bounds; pins never wait, and only a rollback waits for the pins
/// of the publications before it.
#[derive(Default)]
pub struct State {
    published: ArcSwap<Published>,
    /// Serializes lowerings: plugins computing side by side may each lower the publication.
    lowering: Mutex<()>,
}

impl State {
    pub fn pin(&self) -> SafeLengths {
        SafeLengths::new(self.published.load_full())
    }

    /// Publishes `next`, ending with block `tip`, with the address-unspent stores as of `next`
    /// (`None` when the stores hold more).
    pub fn finish_update(&self, next: Lengths, tip: BlockHash, unspent: Option<AddrUnspentViews>) {
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
            unspent,
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
            unspent: None,
            era: Arc::default(),
        }));
        // New readers pin the lowered prefix; the earlier ones finish their reads.
        while Arc::strong_count(&era) > 1 {
            thread::sleep(Duration::from_millis(1));
        }
    }
}
