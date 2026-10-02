use super::*;

fn hashrate_window(len: usize) -> (HashrateSharedData, Vec<Option<StoredU64>>) {
    (
        HashrateSharedData {
            start_day: 30,
            end_day: 30 + len,
            daily_hashrate: vec![Some(StoredF64::from(1_000.0)); len],
            first_heights: (0..len).map(|i| Height::from(i * 10)).collect(),
        },
        (0..len)
            .map(|i| Some(StoredU64::from(i as u64 * 2)))
            .collect(),
    )
}

#[test]
fn skipped_samples_do_not_shift_the_weekly_cadence() {
    for missing in 0..7 {
        let (mut shared, mut cumulative) = hashrate_window(22);
        match missing {
            0 => cumulative[0] = None,
            1 => cumulative[7] = None,
            2 => shared.daily_hashrate[7] = None,
            3 => cumulative[7] = cumulative[0],
            4 => shared.first_heights[7] = shared.first_heights[0],
            5 => cumulative[0] = Some(StoredU64::from(100u64)),
            6 => shared.first_heights[0] = Height::from(100usize),
            _ => unreachable!(),
        }
        let entries: Vec<_> = Query::hashrate_entries(&shared, &cumulative, "pool").collect();
        assert_eq!(entries.len(), if missing == 1 { 1 } else { 2 });
        assert!(
            !entries
                .iter()
                .any(|entry| entry.timestamp == Day1::from(37usize).to_timestamp())
        );
        let last = entries.last().unwrap();
        assert_eq!(last.timestamp, Day1::from(51usize).to_timestamp());
        assert_eq!(last.share, 0.2);
        assert_eq!(last.avg_hashrate, 200);
    }
}
