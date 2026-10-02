use bitview_cohort::AgeRangeId;
use bitview_compute::CohortAccounting;
use brk_types::{BoundedRatio, Cents, CentsSats, CentsSquaredSats, Sats};

#[test]
fn accounting_floors_each_band_before_weighting_and_merges_unrounded_loss_shares() {
    let age = AgeRangeId::Under1H;
    let mut batch = CohortAccounting::default();
    *age.select_mut(&mut batch.supplies) = vec![Sats::new(9), Sats::new(11)];
    *age.select_mut(&mut batch.loss_supplies) = vec![Sats::new(3), Sats::new(7)];
    for raw in [
        99_999_999,
        100_000_000,
        100_000_001,
        199_999_999,
        299_999_999,
    ] {
        *age.select_mut(&mut batch.cap_raw) = vec![CentsSats::new(raw); 2];
        *age.select_mut(&mut batch.capitalized_cap_raw) = vec![CentsSquaredSats::new(raw * raw); 2];
        for encoded in [
            0,
            1,
            BoundedRatio::SCALE / 3,
            BoundedRatio::SCALE / 3 * 2,
            BoundedRatio::SCALE,
        ] {
            let weight = BoundedRatio::from_raw(encoded);
            let actual = batch.weighted(age, 0, weight);
            // Integer arithmetic independently models both separate floor operations.
            let supply = 9 * u64::from(encoded) / u64::from(BoundedRatio::SCALE);
            let cap =
                raw / Sats::ONE_BTC_U128 * u128::from(encoded) / u128::from(BoundedRatio::SCALE);
            assert_eq!(actual.weighted_supply, Sats::new(supply));
            assert_eq!(actual.weighted_cap, Cents::from(cap));
            assert_eq!(
                actual.realized_price(),
                Cents::from(
                    (cap * Sats::ONE_BTC_U128)
                        .checked_div(u128::from(supply))
                        .unwrap_or(0)
                )
            );
            assert_eq!(
                actual.supply_in_loss.value(),
                if encoded == 0 {
                    BoundedRatio::NAN
                } else {
                    BoundedRatio::from(1.0 / 3.0)
                }
            );
        }
    }
    let merged = batch
        .weighted(age, 0, BoundedRatio::from_raw(1234))
        .merged(batch.weighted(age, 1, BoundedRatio::from_raw(9876)));
    let expected = (3 * 1234 + 7 * 9876) as f64 / (9 * 1234 + 11 * 9876) as f64;
    assert_eq!(merged.supply_in_loss.value(), BoundedRatio::from(expected));
}
