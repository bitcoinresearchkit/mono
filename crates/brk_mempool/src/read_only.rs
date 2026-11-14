use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapOption};

use crate::{ReadOnlyState, api::block_template::Template};

/// Cheap reader handle. Each load acquires one complete publication.
#[derive(Clone)]
pub struct ReadOnlyMempool {
    pub(crate) current: Arc<ArcSwap<ReadOnlyState>>,
    /// The newest block template Core accepted, replaced on its own as each is checked
    /// ([`crate::template::Checker`]).
    pub(crate) template: Arc<ArcSwapOption<Template>>,
}

impl ReadOnlyMempool {
    /// Keep this version for the whole query, including deferred body work.
    pub fn load(&self) -> Arc<ReadOnlyState> {
        self.current.load_full()
    }
}
