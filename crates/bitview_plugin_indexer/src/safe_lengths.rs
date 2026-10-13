use std::sync::Arc;

use bitview_primitives::Lengths;
use brk_types::BlockHash;

use crate::state::Published;

/// Pins the published state: the lengths every plugin has computed to, below which rows only
/// change in a rollback. Pinning never waits; a rollback lowers the published lengths and waits
/// for the pins taken before it before changing that prefix.
pub struct SafeLengths(Arc<Published>);

impl SafeLengths {
    pub(crate) fn new(published: Arc<Published>) -> Self {
        Self(published)
    }

    /// Copy the bounds, not their protection. Retain this pin until every
    /// read authorized by these lengths has completed.
    pub fn lengths(&self) -> Lengths {
        self.0.lengths
    }

    /// How many publications came before this one: changes whenever the published state does.
    pub fn revision(&self) -> u64 {
        self.0.revision
    }

    /// The last published block's hash (the default hash while nothing is published).
    pub fn tip(&self) -> BlockHash {
        self.0.tip
    }

    /// The tip of the publication this one followed, or [`Self::tip`] after a rollback.
    pub fn parent_tip(&self) -> BlockHash {
        self.0.parent
    }
}
