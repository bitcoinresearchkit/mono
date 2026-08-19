use bitview_traversable::Traversable;
use brk_types::StoredF32;
use derive_more::Deref;

use bitview_compute::{LazyPerBlock, Windows};

#[derive(Clone, Deref, Traversable)]
pub struct Vecs(
    #[deref]
    #[traversable(flatten)]
    Windows<LazyPerBlock<StoredF32>>,
);

impl Vecs {
    pub fn new(windows: Windows<LazyPerBlock<StoredF32>>) -> Self {
        Self(windows)
    }
}
