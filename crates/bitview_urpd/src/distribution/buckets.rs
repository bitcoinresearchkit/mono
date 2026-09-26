use std::collections::BTreeMap;

use bitview_cohort::AgeRangeId;
use brk_types::{CentsCompact, Sats};

/// Rounding precision for UTXO cost basis prices (5 significant digits in dollars).
pub const COST_BASIS_PRICE_DIGITS: i32 = 5;

pub fn accumulate_masses<T: Default>(
    entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    mut add: impl FnMut(&mut T, AgeRangeId, Sats),
) -> BTreeMap<CentsCompact, T> {
    let mut buckets = BTreeMap::new();
    for (age, price, sats) in entries {
        add(buckets.entry(price).or_default(), age, sats);
    }
    buckets
}

/// Round only after combining all age ranges in each price bucket.
pub fn collect_mass<T>(
    buckets: &BTreeMap<CentsCompact, T>,
    mass: impl Fn(&T) -> f64,
) -> Box<[(CentsCompact, Sats)]> {
    buckets
        .iter()
        .filter_map(|(&price, value)| {
            let mass = mass(value);
            debug_assert!(mass.is_finite() && mass >= 0.0);
            let sats = Sats::from(mass.floor() as u64);
            (sats != Sats::ZERO).then_some((price, sats))
        })
        .collect()
}
