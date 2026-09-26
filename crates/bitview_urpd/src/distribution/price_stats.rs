use brk_types::{Cents, CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN, Sats};

/// Coin- and capital-weighted prices from sorted, rounded URPD buckets.
pub(crate) struct PriceStats {
    pub cost_basis: CostBasisByPercentile,
    pub capitalized_price: Cents,
}

impl PriceStats {
    pub fn from_entries(entries: impl Iterator<Item = (CentsCompact, Sats)> + Clone) -> Self {
        let mut total_sats = 0_u128;
        let mut total_value = 0_u128;
        let mut second_moment = Some(0_u128);
        for (price, sats) in entries.clone() {
            let price = price.as_u128();
            let sats = sats.as_u128();
            let value = price * sats;
            total_sats += sats;
            total_value += value;
            second_moment = second_moment.and_then(|sum| {
                value
                    .checked_mul(price)
                    .and_then(|value| sum.checked_add(value))
            });
        }
        Self {
            cost_basis: percentiles(entries, total_sats, total_value),
            capitalized_price: second_moment
                .and_then(|value| value.checked_div(total_value))
                .map(Cents::from)
                .unwrap_or(Cents::NAN),
        }
    }
}

fn percentiles(
    entries: impl Iterator<Item = (CentsCompact, Sats)>,
    total_sats: u128,
    total_value: u128,
) -> CostBasisByPercentile {
    let per_coin_targets = percentile_targets(total_sats);
    let per_dollar_targets = percentile_targets(total_value);
    let mut prices = CostBasisByPercentile::default();
    let mut per_coin_index = if total_sats == 0 { PERCENTILES_LEN } else { 0 };
    let mut per_dollar_index = if total_value == 0 { PERCENTILES_LEN } else { 0 };
    let mut cumulative_sats = 0_u128;
    let mut cumulative_value = 0_u128;

    for (price, sats) in entries {
        let sats = u128::from(u64::from(sats));
        cumulative_sats += sats;
        cumulative_value += price.as_u128() * sats;
        let price = price.into();
        fill_percentile_prices(
            &mut prices.per_coin,
            &per_coin_targets,
            &mut per_coin_index,
            cumulative_sats,
            price,
        );
        fill_percentile_prices(
            &mut prices.per_dollar,
            &per_dollar_targets,
            &mut per_dollar_index,
            cumulative_value,
            price,
        );
        if per_coin_index == PERCENTILES_LEN && per_dollar_index == PERCENTILES_LEN {
            break;
        }
    }

    prices
}

fn percentile_targets(total: u128) -> [u128; PERCENTILES_LEN] {
    PERCENTILES.map(|percentile| (total * u128::from(percentile) / 100).saturating_sub(1))
}

fn fill_percentile_prices(
    prices: &mut [Cents; PERCENTILES_LEN],
    targets: &[u128; PERCENTILES_LEN],
    target_index: &mut usize,
    cumulative: u128,
    price: Cents,
) {
    while *target_index < PERCENTILES_LEN && cumulative > targets[*target_index] {
        prices[*target_index] = price;
        *target_index += 1;
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
