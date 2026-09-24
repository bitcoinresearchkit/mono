use std::iter;

use brk_types::{CentsCompact, Sats};

use super::COST_BASIS_PRICE_DIGITS;

/// Normalize one age range's sorted prices before applying any weights.
/// Equal rounded prices are combined in integer sats without allocating.
pub fn rounded_entries(
    entries: impl IntoIterator<Item = (CentsCompact, Sats)>,
) -> impl Iterator<Item = (CentsCompact, Sats)> {
    let mut entries = entries
        .into_iter()
        .map(|(price, sats)| (price.round_to_dollar(COST_BASIS_PRICE_DIGITS), sats))
        .peekable();
    iter::from_fn(move || {
        let (price, mut sats) = entries.next()?;
        while let Some((next_price, next_sats)) = entries.peek()
            && *next_price == price
        {
            sats += *next_sats;
            entries.next();
        }
        Some((price, sats))
    })
}

/// Apply one scalar per bucket, flooring weighted supply to whole sats.
pub fn weighted_entries(
    entries: impl IntoIterator<Item = (CentsCompact, Sats)>,
    weight: f64,
) -> impl Iterator<Item = (CentsCompact, Sats)> {
    debug_assert!(weight.is_finite() && (0.0..=1.0).contains(&weight));
    entries.into_iter().filter_map(move |(price, sats)| {
        if weight == 1.0 {
            return Some((price, sats));
        }
        let sats = Sats::from((u64::from(sats) as f64 * weight).floor() as u64);
        (sats != Sats::ZERO).then_some((price, sats))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_weight_floors_each_bucket() {
        let entries = [
            (CentsCompact::new(100), Sats::from(3_u64)),
            (CentsCompact::new(200), Sats::from(1_u64)),
        ];
        assert_eq!(
            weighted_entries(entries, 0.5).collect::<Vec<_>>(),
            [(CentsCompact::new(100), Sats::from(1_u64))],
        );
    }
}
