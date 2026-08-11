use std::ops::Add;

use brk_cohort::{
    ByTerm, ProfitabilityId, ProfitabilityRange, ProfitabilityRangeId, ProfitabilityRow,
};
use brk_types::{Bitcoin, Cents, Dollars, PartsPerMillionSigned32, Sats};
use vecdb::ColumnId;

pub(super) fn sum_terms<T>(rows: &ByTerm<ProfitabilityRange<T>>) -> ProfitabilityRange<T>
where
    T: Add<Output = T> + Copy,
{
    ProfitabilityRange::from_fn(|range| *range.select(&rows.short) + *range.select(&rows.long))
}

pub(super) fn unrealized_pnl_rows(
    spot: Cents,
    cap: &ByTerm<ProfitabilityRange<Dollars>>,
    supply: &ByTerm<ProfitabilityRange<Sats>>,
) -> ByTerm<ProfitabilityRange<Dollars>> {
    ByTerm {
        short: unrealized_pnl_row(spot, &cap.short, &supply.short),
        long: unrealized_pnl_row(spot, &cap.long, &supply.long),
    }
}

fn unrealized_pnl_row(
    spot: Cents,
    cap: &ProfitabilityRange<Dollars>,
    supply: &ProfitabilityRange<Sats>,
) -> ProfitabilityRange<Dollars> {
    ProfitabilityRangeId::from_fn(|column| {
        let market_value =
            f64::from(Dollars::from(spot)) * f64::from(Bitcoin::from(*column.get(supply)));
        let realized_cap = f64::from(*column.get(cap));
        let pnl = if column.is_profit() {
            market_value - realized_cap
        } else {
            realized_cap - market_value
        }
        .max(0.0);
        Dollars::from(pnl)
    })
}

pub(super) fn nupl_row(
    spot: Cents,
    cap: &ProfitabilityRange<Dollars>,
    supply: &ProfitabilityRange<Sats>,
) -> ProfitabilityRow<PartsPerMillionSigned32> {
    let cap = ProfitabilityRow::from_ranges(cap.clone());
    let supply = ProfitabilityRow::from_ranges(supply.clone());
    ProfitabilityId::from_fn(|column| {
        let spot = spot.as_u128();
        let supply = column.get(&supply).as_u128();
        if spot == 0 || supply == 0 {
            PartsPerMillionSigned32::ZERO
        } else {
            let realized_price =
                Cents::from(*column.get(&cap)).as_u128() * Sats::ONE_BTC_U128 / supply;
            PartsPerMillionSigned32::from((spot as f64 - realized_price as f64) / spot as f64)
        }
    })
}

#[cfg(test)]
mod tests {
    use brk_cohort::{
        ByTerm, PROFIT_COUNT, ProfitabilityId, ProfitabilityRangeId, ProfitabilityRow,
    };
    use brk_types::{Cents, Dollars, PartsPerMillionSigned32, Sats};
    use vecdb::ColumnId;

    use super::{nupl_row, sum_terms, unrealized_pnl_rows};

    #[test]
    fn expanded_thresholds_match_prefix_and_suffix_sums() {
        let ranges = ProfitabilityRangeId::from_fn(|id| Sats::from(id.index() as u64 + 1));
        let row = ProfitabilityRow::from_ranges(ranges.clone());
        let sum = |values: &[Sats]| {
            values
                .iter()
                .copied()
                .fold(Sats::default(), |total, value| total + value)
        };

        let ranges: Vec<_> = ranges.iter().copied().collect();
        for (threshold, &column) in ProfitabilityId::profit_ids().iter().enumerate() {
            assert_eq!(
                *column.get(&row),
                sum(&ranges[..PROFIT_COUNT + 1 - threshold])
            );
        }
        for (threshold, &column) in ProfitabilityId::loss_ids().iter().enumerate() {
            assert_eq!(
                *column.get(&row),
                sum(&ranges[PROFIT_COUNT + 1 + threshold..])
            );
        }
    }

    #[test]
    fn derived_rows_preserve_profit_and_loss_polarity() {
        let supply = ProfitabilityRangeId::from_fn(|_| Sats::ONE_BTC);
        let cap = ProfitabilityRangeId::from_fn(|column| {
            Dollars::from(if column.is_profit() { 1.0 } else { 3.0 })
        });
        let spot = Cents::from(200_u64);

        let cap = ByTerm {
            short: cap.clone(),
            long: cap.clone(),
        };
        let supply = ByTerm {
            short: supply.clone(),
            long: supply.clone(),
        };
        let pnl = unrealized_pnl_rows(spot, &cap, &supply);
        let all_cap = sum_terms(&cap);
        let all_supply = sum_terms(&supply);
        let nupl = nupl_row(spot, &all_cap, &all_supply);

        for column in ProfitabilityRangeId::ALL {
            assert_eq!(*column.get(&pnl.short), Dollars::from(1.0));
            assert_eq!(*column.get(&pnl.long), Dollars::from(1.0));
        }
        for column in ProfitabilityId::ALL {
            assert_eq!(
                *column.get(&nupl),
                PartsPerMillionSigned32::from(if column.is_profit() { 0.5 } else { -0.5 })
            );
        }
    }
}
