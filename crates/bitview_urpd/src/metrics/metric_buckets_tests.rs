use std::array;

use bitview_cohort::{AGE_RANGE_COUNT, AgeAggregateId, AgeRange, AgeRangeId};
use brk_types::{CentsCompact, CostBasisByPercentile, PartsPerMillion32, PercentileId, Sats};

use super::*;
use crate::projection::Projection;

fn projected<'a>(
    entries: &'a [(CentsCompact, [u64; AGE_RANGE_COUNT])],
    weights: &AgeRange<f64>,
) -> impl Iterator<Item = ProjectedBucket<1, { AgeAggregateId::ALL.len() }>> + 'a {
    let cohorts: [_; AgeAggregateId::ALL.len()] =
        array::from_fn(|i| AgeAggregateId::ALL[i].age_range_ids());
    let projection = Projection::new(&[Some(weights)], cohorts);
    entries.iter().filter_map(move |(price, supplies)| {
        let occupied = supplies
            .iter()
            .enumerate()
            .fold(0, |mask, (age, &sats)| mask | (u32::from(sats != 0) << age));
        projection.bucket(*price, supplies, occupied)
    })
}

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
            buffer.update(projected(&entries, &weights), spot);
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
            for &id in AgeAggregateId::ALL {
                let expected = project(&|age| id.age_range_ids().contains(&age));
                let mut reference = PriceDistribution::default();
                for (price, sats) in expected {
                    reference.push(price, sats);
                }
                let expected = reference.stats();
                let actual = id.select(&buffer.prices).stats();
                assert_eq!(actual.cost_basis, expected.cost_basis);
                assert_eq!(actual.capitalized_price, expected.capitalized_price);
            }
            for &id in AgeAggregateId::ALL {
                let expected = project(&|age| id.age_range_ids().contains(&age));
                assert_eq!(
                    *id.select(&buffer.density),
                    SupplyDensity::from_entries(expected, spot)
                );
            }
        }
    }
    for (entries, spot, expected, capitalized_price) in [
        (
            &[(94, 10), (95, 20), (100, 30), (105, 40), (106, 100)][..],
            100,
            [0.45, 0.25, 0.2],
            103,
        ),
        (
            &[(95, 10), (96, 10), (106, 10), (107, 10)][..],
            101,
            [0.5, 0.25, 0.25],
            101,
        ),
        (&[(200, 10)][..], 100, [0.0, 0.0, 0.0], 200),
        (
            &[(u32::MAX - 1, 2_100_000_000_000_000)][..],
            u64::from(u32::MAX - 1),
            [1.0, 1.0, 0.0],
            u64::from(u32::MAX - 1),
        ),
    ] {
        let entries: Vec<_> = entries
            .iter()
            .map(|&(price, sats)| {
                let mut supplies = [0; AGE_RANGE_COUNT];
                supplies[AgeRangeId::Under1H.index()] = sats;
                (CentsCompact::new(price), supplies)
            })
            .collect();
        buffer.update(
            projected(&entries, &AgeRange::from_fn(|_| 1.0)),
            Cents::new(spot),
        );
        let actual = &buffer.density.all;
        assert_eq!(
            [actual.total, actual.in_profit, actual.in_loss],
            expected.map(PartsPerMillion32::from)
        );
        assert_eq!(
            buffer.prices.all.stats().capitalized_price,
            Cents::new(capitalized_price)
        );
    }
    let known = [(100, 5), (200, 5)].map(|(price, sats)| {
        let mut supplies = [0; AGE_RANGE_COUNT];
        supplies[AgeRangeId::Under1H.index()] = sats;
        (CentsCompact::new(price), supplies)
    });
    buffer.update(
        projected(&known, &AgeRange::from_fn(|_| 1.0)),
        Cents::new(100),
    );
    let stats = buffer.prices.all.stats();
    for (id, price) in [
        (PercentileId::Pct50, 100),
        (PercentileId::Pct55, 100),
        (PercentileId::Pct60, 200),
    ] {
        assert_eq!(stats.cost_basis.per_coin[id as usize], Cents::new(price));
    }
    assert_eq!(
        stats.cost_basis.per_dollar[PercentileId::Pct50 as usize],
        Cents::new(200)
    );
    assert_eq!(stats.capitalized_price, Cents::new(166));
    buffer.update(
        projected(&entries[..0], &AgeRange::from_fn(|_| 1.0)),
        Cents::new(100),
    );
    assert!(
        buffer
            .prices
            .iter()
            .all(|p| p.stats().capitalized_price.is_nan())
    );
    assert!(
        buffer
            .prices
            .iter()
            .all(|p| p.stats().cost_basis == CostBasisByPercentile::default())
    );
    assert!(buffer.density.iter().all(|d| *d == SupplyDensity::NAN));
}
