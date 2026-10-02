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

#[test]
fn zero_weights_rounding_and_extreme_targets_keep_first_crossing() {
    for weights in [
        vec![0, 0, 0],
        vec![0, 1, 0],
        vec![1, 0, 1],
        vec![1, 2, 0],
        vec![u64::MAX - 1, 0, 1],
    ] {
        let values: Vec<_> = weights
            .into_iter()
            .enumerate()
            .map(|(i, w)| (i as u64, VSize::new(w)))
            .collect();
        let ranks = [
            f64::NEG_INFINITY,
            -1.0,
            0.0,
            0.1,
            0.25,
            0.5,
            0.75,
            1.0,
            2.0,
            f64::INFINITY,
        ];
        assert_eq!(
            get_weighted_percentiles(&values, ranks),
            ranks.map(|p| reference(&values, p))
        );
        assert_eq!(
            get_weighted_percentile(&values, f64::NAN),
            reference(&values, f64::NAN)
        );
        assert_eq!(get_weighted_percentiles(&values, []), [0u64; 0]);
    }
}
