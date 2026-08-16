mod cached_date;
mod cached_first_height;
mod dated;

use brk_traversable::Traversable;
use brk_types::Height;
use vecdb::{ReadableCloneableVec, VecIndex};

pub use self::cached_date::CachedDateVec;
pub use self::cached_first_height::CachedFirstHeightVec;
pub use self::dated::DatedResolutionVecs;

/// Resolution with a pinned, storage-free first-height lookup.
#[derive(Clone, Traversable)]
pub struct ResolutionVecs<I: VecIndex> {
    /// Lowest block height whose resolution index is at least the requested
    /// index. Empty indexes therefore use the first height of the next populated
    /// index; indexes preceding the first populated one resolve to height 0.
    pub first_height: CachedFirstHeightVec<I>,
}

impl<I: VecIndex> ResolutionVecs<I> {
    pub(crate) fn new(mapping: &impl ReadableCloneableVec<Height, I>) -> Self {
        Self {
            first_height: CachedFirstHeightVec::new(mapping.read_only_boxed_clone()),
        }
    }

    pub(crate) fn invalidate_timestamp_caches(&self) {
        self.first_height.invalidate();
    }
}
