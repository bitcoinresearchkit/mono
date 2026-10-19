use brk_types::{Cents, Dollars, Sats};

pub const BINS_PER_DECADE: usize = 200;
const MIN_LOG_BTC: i32 = -8;
const MAX_LOG_BTC: i32 = 4;
pub const NUM_BINS: usize = BINS_PER_DECADE * (MAX_LOG_BTC - MIN_LOG_BTC) as usize;

/// `round(log10(sats) * BINS_PER_DECADE)`, or `None` for zero or beyond the histogram.
#[inline(always)]
pub fn sats_to_bin(sats: Sats) -> Option<usize> {
    if sats.is_zero() {
        return None;
    }
    // At least one sat: the logarithm, so the bin, is nonnegative.
    let bin = ((*sats as f64).log10() * BINS_PER_DECADE as f64).round() as usize;
    (bin < NUM_BINS).then_some(bin)
}

/// Converts a fractional bin to a USD price in cents. At the $100 reference,
/// sats = 100 * 1e8 / P, so P = 10^(10 - bin / 200) dollars.
#[inline]
pub fn bin_to_cents(bin: f64) -> Cents {
    Cents::from(Dollars::from(
        10.0_f64.powf(10.0 - bin / BINS_PER_DECADE as f64),
    ))
}

/// Converts a USD price in cents to a fractional bin, the inverse of [`bin_to_cents`] before
/// its cent rounding; zero maps to positive infinity.
#[inline]
pub fn cents_to_bin(cents: f64) -> f64 {
    (10.0 - (cents / 100.0).log10()) * BINS_PER_DECADE as f64
}
