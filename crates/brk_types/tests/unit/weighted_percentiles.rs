use crate::{VSize, get_weighted_percentile, get_weighted_percentiles};

fn reference(values: &[(u64, VSize)], percentile: f64) -> u64 {
    let total: u64 = values.iter().map(|(_, weight)| u64::from(*weight)).sum();
    let target = (total as f64 * percentile).round() as u64;
    let mut cumulative = 0;
    for &(value, weight) in values {
        cumulative += u64::from(weight);
        if cumulative >= target {
            return value;
        }
    }
    values.last().unwrap().0
}

#[test]
fn batched_ranks_match_independent_scans() {
    let ranks = [0.0, 0.05, 0.10, 0.25, 0.50, 0.50, 0.75, 0.90, 0.95, 1.0];
    for len in [1, 2, 3, 7, 100, 2500] {
        for seed in 0..16 {
            let values: Vec<_> = (0..len)
                .map(|i| (i / 3, VSize::new((i * 13 + seed * 7) % 101)))
                .collect();
            assert_eq!(
                get_weighted_percentiles(&values, ranks),
                ranks.map(|p| reference(&values, p))
            );
            for p in ranks {
                assert_eq!(get_weighted_percentile(&values, p), reference(&values, p));
            }
        }
    }
}
