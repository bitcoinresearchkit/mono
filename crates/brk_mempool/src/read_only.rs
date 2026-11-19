use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapOption};

use crate::ReadOnlyState;

/// Cheap reader handle. Each load acquires one complete publication.
#[derive(Clone)]
pub struct ReadOnlyMempool {
    pub(crate) audits: Arc<ArcSwapOption<crate::audit::AuditReader>>,
    pub(crate) observations: Arc<crate::audit::Observations>,
    pub(crate) current: Arc<ArcSwap<ReadOnlyState>>,
}

impl ReadOnlyMempool {
    pub fn audits(&self) -> Option<Arc<crate::audit::AuditReader>> {
        self.audits.load_full()
    }

    /// Keep this version for the whole query, including deferred body work.
    pub fn load(&self) -> Arc<ReadOnlyState> {
        self.current.load_full()
    }
}
