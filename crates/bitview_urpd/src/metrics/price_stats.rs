#[cfg(test)]
use super::price_distribution::PriceDistribution;

use brk_types::{Cents, CostBasisByPercentile};

#[cfg(test)]
use brk_types::{CentsCompact, Sats};

/// Coin- and capital-weighted prices from sorted, rounded URPD buckets.
pub(super) struct PriceStats {
    pub cost_basis: CostBasisByPercentile,
    pub capitalized_price: Cents,
}

impl Default for PriceStats {
    fn default() -> Self {
        Self {
            cost_basis: CostBasisByPercentile::default(),
            capitalized_price: Cents::NAN,
        }
    }
}

impl PriceStats {
    #[cfg(test)]
    pub fn from_entries(entries: impl Iterator<Item = (CentsCompact, Sats)>) -> Self {
        let mut distribution = PriceDistribution::default();
        for (price, sats) in entries {
            distribution.push(price, sats);
        }
        distribution.stats()
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use brk_types::PercentileId;

    use super::*;

    #[test]
    fn matches_distribution_nearest_rank() {
        let entries = [
            (CentsCompact::new(100), Sats::from(5_u64)),
            (CentsCompact::new(200), Sats::from(5_u64)),
        ];
        let prices = PriceStats::from_entries(entries.into_iter()).cost_basis;
        assert_eq!(
            prices.per_coin[PercentileId::Pct50 as usize],
            Cents::new(100)
        );
        assert_eq!(
            prices.per_coin[PercentileId::Pct55 as usize],
            Cents::new(100)
        );
        assert_eq!(
            prices.per_coin[PercentileId::Pct60 as usize],
            Cents::new(200)
        );
        assert_eq!(
            prices.per_dollar[PercentileId::Pct50 as usize],
            Cents::new(200)
        );
        assert_eq!(
            PriceStats::from_entries(iter::empty()).cost_basis,
            CostBasisByPercentile::default()
        );
    }

    fn price(entries: &[(u32, u64)]) -> Cents {
        PriceStats::from_entries(
            entries
                .iter()
                .map(|&(p, s)| (CentsCompact::new(p), Sats::from(s))),
        )
        .capitalized_price
    }

    #[test]
    fn weights_by_capital_not_coins_or_cohort_means() {
        // Coin-weighted average is 150; capital-weighted is 166.666... cents.
        assert_eq!(price(&[(100, 1), (200, 1)]), Cents::new(166));
        assert_eq!(price(&[(100, 3), (200, 1)]), Cents::new(140));
        // Zero-price and zero-mass buckets do not move either moment.
        assert_eq!(price(&[(0, 999), (100, 1), (200, 0)]), Cents::new(100));
    }

    #[test]
    fn undefined_and_tiny_denominators() {
        assert!(price(&[]).is_nan());
        assert!(price(&[(0, 100), (100, 0)]).is_nan());
        assert_eq!(price(&[(1, 1)]), Cents::new(1));
        assert_eq!(
            price(&[(u32::MAX - 1, 1)]),
            Cents::new(u64::from(u32::MAX - 1))
        );
    }

    #[test]
    fn bitcoin_supply_at_largest_bucket_fits_without_float_rounding() {
        assert_eq!(
            price(&[(u32::MAX - 1, 2_100_000_000_000_000)]),
            Cents::new(u64::from(u32::MAX - 1))
        );
    }

    #[test]
    fn second_moment_overflow_does_not_discard_percentiles() {
        let entries = [
            (CentsCompact::new(u32::MAX - 2), Sats::from(u64::MAX)),
            (CentsCompact::new(u32::MAX - 1), Sats::from(u64::MAX)),
        ];
        let stats = PriceStats::from_entries(entries.into_iter());
        assert!(stats.capitalized_price.is_nan());
        assert_eq!(
            stats.cost_basis.per_coin[PercentileId::Pct50 as usize],
            Cents::new(u64::from(u32::MAX - 2))
        );
        assert_eq!(
            stats.cost_basis.per_dollar[PercentileId::Pct50 as usize],
            Cents::new(u64::from(u32::MAX - 1))
        );
    }
}
