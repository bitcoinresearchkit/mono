//! Price windows that never reach into the zero-price prefix before the first price: they
//! clamp to start there, and rows before it have no window.

use bitview_plugin_price::{first_priced, priced_range};
use bitview_vecs::LazyWindowStartVec;
use brk_types::{Cents, Height, Version};
use vecdb::{
    AnyVec, DeltaOp, LazyDeltaVec, LazyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec,
    UnaryTransform, VecIndex,
};

/// A price SMA over a cumulative price source and plain window starts.
pub type PricedSmaVec = LazyDeltaVec<Height, Cents, Cents, PricedSma>;

/// Average of the window's priced rows, rounded to the nearest cent; NaN before the first price.
/// The prefix sum before a window start in the zero prefix equals the one before the first price.
pub struct PricedSma;

impl DeltaOp<Cents, Cents> for PricedSma {
    fn count(index: usize, start: usize) -> usize {
        let priced = priced_range(start..index + 1);
        priced.end - priced.start
    }
    fn combine(current: Cents, before: Cents, count: usize) -> Cents {
        if count == 0 {
            Cents::NAN
        } else {
            let count = count as u64;
            Cents::new((*current - *before + count / 2) / count)
        }
    }
}

/// Window starts clamped to the first priced height.
pub struct FromFirstPrice;

impl UnaryTransform<Height, Height> for FromFirstPrice {
    #[inline(always)]
    fn apply(start: Height) -> Height {
        Height::from(first_priced(start.to_usize()))
    }

    /// One bulk read: window starts locate their first start once per read, so the default
    /// chunked read would search the timestamp history again for every chunk.
    fn read_into<I: VecIndex>(
        source: &ReadableBoxedVec<I, Height>,
        from: usize,
        to: usize,
        out: &mut Vec<Height>,
    ) {
        let first = out.len();
        source.read_into_at(from, to, out);
        for start in &mut out[first..] {
            *start = Self::apply(*start);
        }
    }
}

/// `starts` clamped to the first priced height: rows before it get an empty window.
pub fn priced_starts(starts: &LazyWindowStartVec) -> LazyVec<Height, Height, Height> {
    LazyVec::transformed::<FromFirstPrice>(
        &format!("{}_priced", starts.name()),
        starts.version() + Version::ONE,
        starts.read_only_boxed_clone(),
    )
}
