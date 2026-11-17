use bitview_primitives::CentsCompact;

use crate::COST_BASIS_PRICE_DIGITS;

/// Prices below this many cents aren't rounded.
const EXACT_PRICES: u32 = 10u32.pow(COST_BASIS_PRICE_DIGITS);
/// The rounded prices a decade above those holds.
const PRICES_PER_DECADE: u32 = EXACT_PRICES - EXACT_PRICES / 10;
/// Every price rounding can produce, a place each.
pub(crate) const PRICE_PLACES: usize = (EXACT_PRICES + PRICES_PER_DECADE * (u32::MAX.ilog10() + 1 - COST_BASIS_PRICE_DIGITS)) as usize;

/// A rounded price's place among all the prices rounding can produce, in ascending price order.
#[inline]
pub(crate) fn price_place(rounded: CentsCompact) -> usize {
    let cents = rounded.inner();
    if cents < EXACT_PRICES {
        return cents as usize;
    }
    let decade = cents.ilog10() - COST_BASIS_PRICE_DIGITS;
    let leading = cents / 10u32.pow(decade + 1);
    (EXACT_PRICES + PRICES_PER_DECADE * decade + leading - EXACT_PRICES / 10) as usize
}

/// The rounded price at `place`: `price_place`'s inverse over the places a price reaches.
#[inline]
pub(crate) fn place_price(place: usize) -> CentsCompact {
    let place = place as u32;
    if place < EXACT_PRICES {
        return CentsCompact::new(place);
    }
    let decade = (place - EXACT_PRICES) / PRICES_PER_DECADE;
    let leading = (place - EXACT_PRICES) % PRICES_PER_DECADE + EXACT_PRICES / 10;
    CentsCompact::new(leading * 10u32.pow(decade + 1))
}
