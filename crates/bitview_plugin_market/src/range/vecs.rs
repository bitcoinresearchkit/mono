use bitview_compute::{ExtremaWindow, Retained};
use bitview_traversable::Traversable;
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use super::price_min_max_vecs::PriceMinMaxVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Minimum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block. Windows start no earlier than the
    /// first exchange price (block 74,641); null before it.
    pub min: PriceMinMaxVecs<M>,
    /// Maximum block-level Bitcoin spot price in a trailing monotonic-time
    /// window, including the represented block. Windows start no earlier than the
    /// first exchange price (block 74,641); null before it.
    pub max: PriceMinMaxVecs<M>,
    pub(super) extrema: M::WriteOnly<[Retained<ExtremaWindow<Cents>>; 3]>,
}
