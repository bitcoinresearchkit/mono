use bitview_cohort::AgeRangeId;
use bitview_compute::{CohortAccounting, WeightedCohortState};
use brk_types::{BoundedRatio, Cents, CentsSats, CentsSquaredSats, Sats};

#[test]
fn accounting_floors_each_band_before_weighting() {
    let age = AgeRangeId::ALL[0];
    let mut batch = CohortAccounting::default();
    for raw in [99_999_999, 100_000_000, 100_000_001, 299_999_999] {
        *age.select_mut(&mut batch.supplies) = vec![Sats::new(9)];
        *age.select_mut(&mut batch.loss_supplies) = vec![Sats::new(3)];
        *age.select_mut(&mut batch.cap_raw) = vec![CentsSats::new(raw)];
        *age.select_mut(&mut batch.capitalized_cap_raw) = vec![CentsSquaredSats::new(raw * 3)];
        for value in [0.0, 1.0 / 3.0, 1.0] {
            let weight = BoundedRatio::from(value);
            let actual = batch.weighted(age, 0, weight);
            let mut expected = WeightedCohortState::default();
            expected.add(
                Sats::new(9),
                Sats::new(3),
                Cents::new((raw / Sats::ONE_BTC_U128) as u64),
                weight,
            );
            assert_eq!(actual.weighted_cap, expected.weighted_cap);
            assert_eq!(actual.weighted_supply, expected.weighted_supply);
            assert_eq!(actual.realized_price(), expected.realized_price());
        }
    }
}
