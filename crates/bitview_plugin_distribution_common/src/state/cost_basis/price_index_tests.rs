use super::*;

#[test]
fn bulk_and_incremental_queries_match_a_sorted_reference() {
    let rows = [
        (0, 7, true),
        (101, 13, true),
        (1_049, 19, false),
        (9_999_949, 23, true),
        (10_000_501, 31, false),
        (99_999_501, 37, true),
        (100_000_000, 41, false),
        (200_000_000, 43, true),
    ]
    .map(|(price, sats, sth)| (CentsCompact::from(Cents::new(price)), sats, sth));
    let mut bulk = PriceIndex::<2>::default();
    let mut incremental = PriceIndex::<2>::default();
    for &(price, sats, sth) in &rows {
        bulk.add_raw(price, sats, [true, sth]);
        incremental.add(price, sats, [true, sth]);
    }
    bulk.build();
    let batches = [&bulk, &incremental].map(|index| {
        index.percentiles::<3>(|filter, n| match filter {
            0 => (n.sats[0], n.cap[0]),
            1 => (n.sats[1], n.cap[1]),
            _ => (n.sats[0] - n.sats[1], n.cap[0] - n.cap[1]),
        })
    });
    for filter in 0..3 {
        let included = |sth| filter == 0 || (filter == 1) == sth;
        let selected: Vec<_> = rows.iter().filter(|&&(_, _, sth)| included(sth)).collect();
        let total_sats = selected.iter().map(|&&(_, s, _)| s).sum::<i64>();
        let total_cap = selected
            .iter()
            .map(|&&(p, s, _)| p.as_u128() as i128 * s as i128)
            .sum::<i128>();
        let expected = |target: i128, weighted: bool| {
            let mut sum = 0;
            for &&(price, sats, _) in &selected {
                sum += if weighted {
                    price.as_u128() as i128 * sats as i128
                } else {
                    sats as i128
                };
                if sum > target {
                    return bucket_to_cents(cents_to_bucket(price.into()));
                }
            }
            panic!("unreachable percentile target");
        };
        for batch in &batches {
            let result = &batch[filter];
            assert_eq!(result.min_price, expected(0, false));
            assert_eq!(result.max_price, expected(total_sats as i128 - 1, false));
            for (i, &p) in PERCENTILES.iter().enumerate() {
                assert_eq!(
                    result.sat_prices[i],
                    expected((total_sats as i128 * p as i128 / 100 - 1).max(0), false)
                );
                assert_eq!(
                    result.usd_prices[i],
                    expected((total_cap * p as i128 / 100 - 1).max(0), true)
                );
            }
        }
    }
    for price in [
        0,
        100,
        1000,
        9_999_900,
        10_000_000,
        99_999_999,
        100_000_000,
        200_000_000,
    ] {
        let price = Cents::new(price);
        let expected = rows
            .iter()
            .filter(|&&(p, _, _)| cents_to_bucket(p.into()) < cents_to_bucket(price))
            .fold(PriceTotals::<2>::default(), |mut result, &(p, s, sth)| {
                result.add_assign(&PriceIndex::delta(p, s, [true, sth]));
                result
            });
        assert_eq!(bulk.before(price).sats, expected.sats);
        assert_eq!(bulk.before(price).cap, expected.cap);
        assert_eq!(bulk.before(price).sats, incremental.before(price).sats);
        assert_eq!(
            bulk.density_range(price).sats,
            incremental.density_range(price).sats
        );
    }
    for &(price, sats, sth) in &rows {
        bulk.add(price, -sats, [true, sth]);
    }
    assert_eq!(bulk.totals().sats, [0; 2]);
    assert_eq!(bulk.totals().cap, [0; 2]);
    assert_eq!(
        bulk.percentiles::<1>(|_, n| (n.sats[0], n.cap[0]))[0].max_price,
        Cents::ZERO
    );
}
