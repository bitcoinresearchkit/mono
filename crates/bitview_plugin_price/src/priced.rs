//! Spot-price coverage: heights before the first exchange price (block 74,641) hold zero
//! placeholders, not prices. Each consumer keeps its own policy for the unpriced ones.

use std::ops::Range;

use brk_oracle::FIRST_PRICE_HEIGHT;

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
