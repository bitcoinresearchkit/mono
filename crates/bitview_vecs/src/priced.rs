//! Spot-price coverage and dollar windows. Heights before the first exchange price (block
//! 74,641) hold zero placeholders, not prices: a dollar value there is zero. A trailing window
//! reaching into them would read short (a sum, a change from zero), so a dollar window has no
//! value (NaN) until it's priced ([`PricedWindow`]); a mean of per-block values is over the
//! window's priced blocks ([`PricedRows`]), and a window may start at the first price instead
//! ([`priced_starts`]).

use std::{marker::PhantomData, ops::Range};

use bitview_compute::Quantity;
use brk_oracle::FIRST_PRICE_HEIGHT;
use brk_types::{Height, Version};
use vecdb::{
    AnyVec, DeltaAvg, DeltaChange, DeltaOp, DeltaSub, LazyVec, ReadableCloneableVec, ReadableVec,
    VecIndex,
};

use crate::{LazyDeltaFromHeight, LazyWindowStartVec};

/// Added to a dollar window's version: what's read from it changed (NaN where it reaches before
/// the first price), so whatever is computed from it is again.
pub const PRICED_VERSION: Version = Version::ONE;

#[inline]
pub fn is_priced(height: usize) -> bool {
    height >= FIRST_PRICE_HEIGHT
}

/// The first priced height at or after `height`.
#[inline]
pub fn first_priced(height: usize) -> usize {
    height.max(FIRST_PRICE_HEIGHT)
}

/// The priced heights of `heights`, a suffix of them: empty at its end when none is priced.
#[inline]
pub fn priced_range(heights: Range<usize>) -> Range<usize> {
    first_priced(heights.start).min(heights.end)..heights.end
}

/// The first height whose window (`starts`, never decreasing) starts at or after the first price:
/// the first a dollar sum over it has a value (their length when none does yet).
pub fn first_wholly_priced(starts: &impl ReadableVec<Height, Height>) -> usize {
    let (mut low, mut high) = (0, starts.len());
    while low < high {
        let middle = (low + high) / 2;
        if is_priced(starts.collect_one_at(middle).unwrap().to_usize()) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// `Op` over a priced window: its blocks, and with `BEFORE` the value just before it too (a
/// level's change is from it; a sum's is a cumulative's, zero through the unpriced blocks); NaN
/// otherwise (its count 0: a window always holds its own block, so `Op` never counts 0 itself).
#[derive(Clone, Copy)]
pub struct PricedWindow<Op, const BEFORE: bool>(PhantomData<fn() -> Op>);

impl<S, T, Op, const BEFORE: bool> DeltaOp<S, T> for PricedWindow<Op, BEFORE>
where
    T: From<f64>,
    Op: DeltaOp<S, T>,
{
    #[inline]
    fn count(h: usize, start: usize) -> usize {
        if start >= FIRST_PRICE_HEIGHT + usize::from(BEFORE) {
            Op::count(h, start)
        } else {
            0
        }
    }

    #[inline]
    fn combine(current: S, before: S, count: usize) -> T {
        priced_combine::<S, T, Op>(current, before, count)
    }
}

/// `Op` over the window's priced blocks (counted from the first price on); NaN for a window with
/// none. For a cumulative of dollar values, zero through the unpriced blocks: the difference over
/// the window is the priced blocks' alone.
#[derive(Clone, Copy)]
pub struct PricedRows<Op>(PhantomData<fn() -> Op>);

impl<S, T, Op> DeltaOp<S, T> for PricedRows<Op>
where
    T: From<f64>,
    Op: DeltaOp<S, T>,
{
    #[inline]
    fn count(h: usize, start: usize) -> usize {
        priced_range(start..h + 1).len()
    }

    #[inline]
    fn combine(current: S, before: S, count: usize) -> T {
        priced_combine::<S, T, Op>(current, before, count)
    }
}

/// A priced op's value: none (NaN) without a count.
#[inline]
fn priced_combine<S, T: From<f64>, Op: DeltaOp<S, T>>(current: S, before: S, count: usize) -> T {
    if count == 0 {
        T::from(f64::NAN)
    } else {
        Op::combine(current, before, count)
    }
}

/// A dollar sum's op: NaN until its window starts at or after the first price.
pub type PricedSum = PricedWindow<DeltaSub, false>;
/// A dollar level's change's op: NaN until its window starts after the first price.
pub type PricedChange = PricedWindow<DeltaChange, true>;

/// A dollar rolling sum: NaN until its window is wholly priced.
pub type PricedRollingSumFromHeight<T> = LazyDeltaFromHeight<T, T, PricedSum>;
/// A dollar rolling mean of per-block values: over the window's priced blocks.
pub type PricedRollingAvgFromHeight<T> =
    LazyDeltaFromHeight<T, <T as Quantity>::Fract, PricedRows<DeltaAvg>>;
/// A dollar level's change over a window: NaN until it and the value before it are priced.
pub type PricedDeltaFromHeight<S, C> = LazyDeltaFromHeight<S, C, PricedChange>;

/// `starts` clamped to the first priced height, and at most just past their own row: a row
/// before the first price gets an empty window (an op over it counts no priced rows:
/// [`PricedRows`]), whose value before it is its own row, read with it.
pub fn priced_starts(starts: &LazyWindowStartVec) -> LazyVec<Height, Height, Height> {
    LazyVec::new(
        &format!("{}_priced", starts.name()),
        starts.version() + Version::ONE,
        starts.read_only_boxed_clone(),
        |height, start| Height::from(first_priced(start.to_usize()).min(height.to_usize() + 1)),
    )
}
