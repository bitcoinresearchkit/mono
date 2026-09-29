use bitview_cohort::UTXOAggregateId;

use super::*;
use crate::metrics::price_stats::PriceStats;

#[test]
fn fused_statistics_match_independent_cohort_projection_and_density() {
    let entries: Vec<_> = [0, 94, 95, 96, 100, 105, 106, u32::MAX - 1]
        .into_iter()
        .enumerate()
        .map(|(i, price)| {
            (
                CentsCompact::new(price),
                array::from_fn(|age| ((age * 11 + i * 3) % 17) as u64),
            )
        })
        .collect();
    let mut buffer = MetricBuckets::default();
    for spot in [
        Cents::ZERO,
        Cents::NAN,
        Cents::new(100),
        Cents::new(101),
        Cents::new(u64::from(u32::MAX - 1)),
    ] {
        for offset in 0..5 {
            let weights = AgeRange::from_fn(|age| match offset {
                0 => 0.0,
                1 => 1.0,
                2 => 0.5,
                _ => ((age.index() * 7 + offset * 3) % 17) as f64 / 17.0,
            });
            buffer.update(entries.iter().map(|(p, s)| (*p, s)), &weights, spot);
            let project = |accept: &dyn Fn(AgeRangeId) -> bool| {
                entries
                    .iter()
                    .filter_map(|(price, supplies)| {
                        let sats = AgeRangeId::ALL
                            .iter()
                            .filter(|&&age| accept(age))
                            .map(|&age| supplies[age.index()] as f64 * *age.select(&weights))
                            .sum::<f64>()
                            .floor() as u64;
                        (sats != 0).then_some((*price, Sats::new(sats)))
                    })
                    .collect::<Vec<_>>()
            };
            for &id in UTXOAggregateId::ALL {
                let expected = project(&|age| id.age_range_ids().contains(&age));
                let expected = PriceStats::from_entries(expected.into_iter());
                let actual = id.select(&buffer.prices).stats();
                assert_eq!(actual.cost_basis, expected.cost_basis);
                assert_eq!(actual.capitalized_price, expected.capitalized_price);
            }
            for (i, excluded) in [
                None,
                Some(AgeRangeId::From4MTo5M),
                Some(AgeRangeId::From5MTo6M),
                Some(AgeRangeId::From6MTo9M),
            ]
            .into_iter()
            .enumerate()
            {
                let expected =
                    project(&|age| excluded.is_none_or(|excluded| age.index() < excluded.index()));
                assert_eq!(
                    buffer.density[i],
                    SupplyDensity::from_entries(expected, spot)
                );
            }
        }
    }
    buffer.update(
        entries[..0].iter().map(|(p, s)| (*p, s)),
        &AgeRange::from_fn(|_| 1.0),
        Cents::new(100),
    );
    assert!(
        buffer
            .prices
            .iter()
            .all(|p| p.stats().capitalized_price.is_nan())
    );
    assert_eq!(buffer.density, [SupplyDensity::NAN; 4]);
}
