use serde_json::to_value;

use super::*;

#[test]
fn completed_closes_are_causal_and_share_missing_data_rules() {
    let mapping = [0usize, 2, 2, 3, 4].map(Height::from);
    let values = [
        Cents::new(100),
        Cents::new(200),
        Cents::ZERO,
        Cents::new(400),
    ];
    let query = |target| {
        historical_prices(&mapping, values.len(), target, |height| {
            Ok(values[usize::from(height)])
        })
        .unwrap()
    };
    let all = query(None);
    assert_eq!(all.prices.len(), 3);
    assert_eq!(
        all.prices.iter().map(|p| *p.time).collect::<Vec<_>>(),
        [1, 3, 4].map(|i| INDEX_EPOCH + i * HOUR4_INTERVAL)
    );
    assert_eq!(all.prices[1].usd, Dollars::from(Cents::ZERO));
    for timestamp in [0, INDEX_EPOCH, INDEX_EPOCH + HOUR4_INTERVAL - 1] {
        assert!(query(Some(Timestamp::new(timestamp))).prices.is_empty());
    }
    for timestamp in [
        INDEX_EPOCH + HOUR4_INTERVAL,
        INDEX_EPOCH + 2 * HOUR4_INTERVAL,
        INDEX_EPOCH + 3 * HOUR4_INTERVAL,
        u32::MAX,
    ] {
        let point = query(Some(Timestamp::new(timestamp)));
        let expected = all
            .prices
            .iter()
            .rev()
            .find(|p| *p.time <= timestamp)
            .unwrap();
        assert_eq!(
            to_value(&point.prices[0]).unwrap(),
            to_value(expected).unwrap()
        );
    }
    assert!(
        historical_prices(&[Height::ZERO], 1, None, |_| panic!(
            "partial tail must not read prices"
        ))
        .unwrap()
        .prices
        .is_empty()
    );
}
