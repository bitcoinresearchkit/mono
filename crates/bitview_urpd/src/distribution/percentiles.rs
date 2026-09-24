use brk_types::{Cents, CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN, Sats};

/// Coin- and capital-weighted percentiles from sorted price buckets.
pub fn cost_basis_percentiles(
    entries: impl Iterator<Item = (CentsCompact, Sats)> + Clone,
) -> CostBasisByPercentile {
    let (total_sats, total_value) = entries.clone().fold(
        (0_u128, 0_u128),
        |(total_sats, total_value), (price, sats)| {
            let sats = u128::from(u64::from(sats));
            (total_sats + sats, total_value + price.as_u128() * sats)
        },
    );
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
        let prices = cost_basis_percentiles(entries.into_iter());
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
            cost_basis_percentiles(iter::empty()),
            CostBasisByPercentile::default()
        );
    }
}
