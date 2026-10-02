use brk_types::Sats;

pub const BINS_PER_DECADE: usize = 200;
const MIN_LOG_BTC: i32 = -8;
const MAX_LOG_BTC: i32 = 4;
pub const NUM_BINS: usize = BINS_PER_DECADE * (MAX_LOG_BTC - MIN_LOG_BTC) as usize;

/// Maps a satoshi value to its log-scale bin index.
/// bin = round(log10(sats) * BINS_PER_DECADE).
#[inline(always)]
pub fn sats_to_bin(sats: Sats) -> Option<usize> {
    if sats.is_zero() {
        return None;
    }
    let bin = ((*sats as f64).log10() * BINS_PER_DECADE as f64).round() as i64;
    if bin >= 0 && (bin as usize) < NUM_BINS {
        Some(bin as usize)
    } else {
        None
    }
}

/// Converts a fractional bin to a USD price in cents.
/// For a $D output at price P: sats = D * 1e8 / P, so P = 10^(10 - bin/200) dollars,
/// where 10 = log10($100 reference * 1e8 sats/BTC).
#[inline]
pub fn bin_to_cents(bin: f64) -> u64 {
    let dollars = 10.0_f64.powf(10.0 - bin / BINS_PER_DECADE as f64);
    (dollars * 100.0).round() as u64
}

/// Converts a USD price in cents to a fractional bin (inverse of bin_to_cents).
#[inline]
pub fn cents_to_bin(cents: f64) -> f64 {
    (10.0 - (cents / 100.0).log10()) * BINS_PER_DECADE as f64
}
