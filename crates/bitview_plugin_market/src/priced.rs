//! Price windows that never reach into the zero-price prefix before
//! [`FIRST_PRICE_HEIGHT`]: they clamp to start there, and rows before it have no window.

use bitview_plugin_price::FIRST_PRICE_HEIGHT;
use bitview_vecs::LazyWindowStartVec;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyVec, DeltaOp, LazyDeltaVec, LazyVec, ReadableCloneableVec, UnaryTransform};

/// A price SMA over a cumulative price source and plain window starts.
pub type PricedSmaVec = LazyDeltaVec<Height, Cents, Cents, PricedSma>;

/// Integer-cent average of the window's priced rows; NaN before the first price. The prefix
/// sum before a window start in the zero prefix equals the one before the first price.
pub struct PricedSma;

impl DeltaOp<Cents, Cents> for PricedSma {
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }
    fn ago_default() -> Cents {
        Cents::ZERO
    }
    fn count(index: usize, start: usize) -> usize {
        (index + 1).saturating_sub(start.max(FIRST_PRICE_HEIGHT))
    }
    fn combine(current: Cents, previous: Cents, count: usize) -> Cents {
        if count == 0 {
            Cents::NAN
        } else {
            Cents::new((*current - *previous) / count as u64)
        }
    }
}

/// Window starts clamped to the first priced height.
pub struct FromFirstPrice;

impl UnaryTransform<Height, Height> for FromFirstPrice {
    #[inline(always)]
    fn apply(start: Height) -> Height {
        start.max(Height::from(FIRST_PRICE_HEIGHT))
    }
}

/// `starts` clamped to the first priced height: rows before it get an empty window.
pub fn priced_starts(starts: &LazyWindowStartVec) -> LazyVec<Height, Height, Height, Height> {
    LazyVec::transformed::<FromFirstPrice>(
        &format!("{}_priced", starts.name()),
        starts.version() + Version::ONE,
        starts.read_only_boxed_clone(),
    )
}
