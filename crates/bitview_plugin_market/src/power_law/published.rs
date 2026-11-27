//! The published lines, each a closed formula of the block's day (UTC) with its author's constants.

use bitview_primitives::Day1;
use brk_types::{Cents, Height};

/// The genesis block's day, 2009-01-03, as a `Day1` (which counts from 2009-01-01): the authors
/// count whole days from it.
const GENESIS: u16 = 2;
/// 2012-01-01, where the Rainbow chart's days start.
const RAINBOW_START: u16 = 1095;

/// Whole days from the genesis block's day to `day`.
pub(super) fn days_since_genesis(day: Day1) -> f64 {
    u64::from(day) as f64 - f64::from(GENESIS)
}

/// `log10(price in dollars)` of a power law in days since genesis, in cents; null on and before
/// the genesis day, where the law has no value.
fn power_law(day: Day1, intercept: f64, slope: f64) -> Cents {
    let days = days_since_genesis(day);
    if days <= 0.0 {
        return Cents::NAN;
    }
    Cents::from(10f64.powf(intercept + slope * days.log10() + 2.0))
}

/// Burger's power law: its intercept and slope, and how far under it (in log10) his support line sits.
const BURGER: (f64, f64) = (-17.015_933_13, 5.845_093_76);
const BURGER_SUPPORT: f64 = 0.45;

pub(super) fn burger(_: Height, day: Day1) -> Cents {
    power_law(day, BURGER.0, BURGER.1)
}

pub(super) fn burger_floor(_: Height, day: Day1) -> Cents {
    power_law(day, BURGER.0 - BURGER_SUPPORT, BURGER.1)
}

pub(super) fn mooij(_: Height, day: Day1) -> Cents {
    power_law(day, -16.493 + 0.42f64.log10(), 5.688)
}

pub(super) fn rainbow_lowest_band(_: Height, day: Day1) -> Cents {
    let Some(days) = u64::from(day).checked_sub(u64::from(RAINBOW_START)) else {
        return Cents::NAN;
    };
    Cents::from(10f64.powf(2.775 * (days as f64 + 1175.0).ln() - 19.463 + 2.0))
}
