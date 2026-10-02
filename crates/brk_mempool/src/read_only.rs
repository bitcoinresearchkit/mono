use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::ReadOnlyState;

/// Cheap reader handle. Each load acquires one complete publication.
///
#[derive(Clone)]
pub struct ReadOnlyMempool {
    pub(crate) current: Arc<ArcSwap<ReadOnlyState>>,
}

impl ReadOnlyMempool {
    /// Keep this version for the whole query, including deferred body work.
    pub fn load(&self) -> Arc<ReadOnlyState> {
        self.current.load_full()
    }
}
