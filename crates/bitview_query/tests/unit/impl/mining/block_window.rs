use std::collections::BTreeMap;

use super::*;

#[test]
fn hashed_buckets_match_ordered_groups_for_all_periods() {
    let timestamps: Vec<_> = (0..10_000u32)
        .map(|height| {
            Timestamp::from(1_231_006_505 + height * 600 - if height % 11 == 0 { 1200 } else { 0 })
        })
        .chain([Timestamp::from(u32::MAX), Timestamp::from(0u32)])
        .collect();
    for period in [
        TimePeriod::Day,
        TimePeriod::ThreeDays,
        TimePeriod::Week,
        TimePeriod::Month,
        TimePeriod::ThreeMonths,
        TimePeriod::SixMonths,
        TimePeriod::Year,
        TimePeriod::TwoYears,
        TimePeriod::ThreeYears,
        TimePeriod::All,
    ] {
        let div = time_div(period);
        let mut expected: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for (i, timestamp) in timestamps.iter().enumerate() {
            expected.entry(**timestamp / div).or_default().push(i);
        }
        assert_eq!(
            bucket_offsets(&timestamps, div),
            expected.into_iter().collect::<Vec<_>>()
        );
        assert!(bucket_offsets(&[], div).is_empty());
    }
}
