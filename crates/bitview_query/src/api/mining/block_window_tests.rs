use super::*;
use std::collections::BTreeMap;

fn range(from: u32, to: u32) -> TimestampRange {
    TimestampRange::new(Timestamp::new(from), Timestamp::new(to)).unwrap()
}

// Independent source fixture: directly sort each trailing raw-time window,
// including the upper middle for short even windows. No bound implementation.
fn medians(raw: &[u32]) -> Vec<u32> {
    (0..raw.len()).map(|height| {
        let mut window = raw[height.saturating_sub(10)..=height].to_vec();
        window.sort_unstable();
        window[window.len() / 2]
    }).collect()
}

fn assert_window(raw: &[u32], from: u32, to: u32) {
    let range = range(from, to);
    let median = medians(raw);
    let mut max = 0;
    let maximum: Vec<_> = raw.iter().map(|&time| {
        max = max.max(time);
        max
    }).collect();
    let bounds = timestamp_candidates(raw.len(), range,
        |height| Ok(Timestamp::new(maximum[height])),
        |height| Ok(Timestamp::new(median[height])),
    ).unwrap();
    let div = span_div(range.span());
    let bw = BlockWindow::select(bounds, div, |time| range.contains(time),
        |height| Ok(Timestamp::new(raw[height])),
    ).unwrap();

    // Full-column oracle does not use prefix max, MTP, candidate bounds or contains().
    let expected_heights: Vec<_> = raw.iter().enumerate()
        .filter(|(_, time)| from <= **time && **time <= to)
        .map(|(height, _)| height).collect();
    assert_eq!(bw.selected.iter().map(|(h, _)| usize::from(*h)).collect::<Vec<_>>(),
        expected_heights, "raw={raw:?}, [{from},{to}]");
    let fees: Vec<u64> = (0..raw.len()).map(|h| (h as u64 * 7 + 1) % 101).collect();
    let sums = bw.fold_with(0u128, |sum, fee: u64| *sum += u128::from(fee),
        |height| Ok(fees[usize::from(height)]),
    ).unwrap();
    let mut expected = BTreeMap::<u32, Vec<usize>>::new();
    for height in expected_heights {
        expected.entry(raw[height] / div).or_default().push(height);
    }
    assert_eq!(bw.buckets.len(), expected.len());
    // Equivalent arithmetic written independently from round_half_up().
    let mean = |sum: u128, n: usize| ((2 * sum + n as u128) / (2 * n as u128)) as u64;
    for ((bucket, sum), heights) in bw.buckets.iter().zip(sums).zip(expected.values()) {
        assert_eq!(u64::from(bucket.avg_height), mean(heights.iter().map(|&h| h as u128).sum(), heights.len()));
        assert_eq!(u64::from(bucket.avg_timestamp), mean(heights.iter().map(|&h| u128::from(raw[h])).sum(), heights.len()));
        assert_eq!(round_half_up(sum, bucket.count.into()), mean(heights.iter().map(|&h| u128::from(fees[h])).sum(), heights.len()));
        assert_eq!(bucket.count as usize, heights.len());
    }
}

#[test]
fn bounded_timestamp_selection_matches_full_filter() {
    // All extensions satisfy raw[h] > MTP[h-1]. The future outlier is placed
    // after the early even windows, so low descendants remain consensus-valid.
    let mut raw: Vec<u32> = (100..112).collect();
    raw.push(1_000_000);
    for height in 13..80 {
        let median = *medians(&raw).last().unwrap();
        raw.push(median + 1 + if height % 17 == 0 { 500 } else { 0 });
    }
    let median = medians(&raw);
    assert!(raw.windows(2).any(|w| w[0] > w[1]));
    assert!(raw.iter().enumerate().any(|(h, time)| raw[..h].contains(time)));
    assert!(median.windows(2).any(|w| w[0] == w[1]));
    assert!(median.is_sorted());
    assert!((1..raw.len()).all(|height| raw[height] > median[height - 1]));

    // Every prefix exercises genesis and each of the first eleven conventions,
    // first upper crossing, duplicate boundaries, no matches, and a skew plateau.
    for len in 0..=raw.len() {
        let chain = &raw[..len];
        let mut bounds = vec![0, 1, 99, 100, 101, 112, 999_999, 1_000_000, 1_000_001, u32::MAX];
        for &time in chain {
            bounds.extend([time - 1, time, time + 1]);
        }
        bounds.sort_unstable();
        bounds.dedup();
        for (i, &from) in bounds.iter().enumerate() {
            for &to in &bounds[i..] {
                assert_window(chain, from, to);
            }
        }
    }
    for chain in [vec![0], vec![0, u32::MAX - 2, u32::MAX - 1, u32::MAX]] {
        for (from, to) in [(0, 0), (0, u32::MAX), (1, 1),
            (u32::MAX - 1, u32::MAX - 1), (u32::MAX, u32::MAX)] {
            assert_window(&chain, from, to);
        }
    }
    // Coarser grouping, unsorted bucket visits, equal raw timestamps in different heights.
    let mut large: Vec<u32> = (0..12).map(|h| 1_000 + h * 600).collect();
    for height in 12..100 {
        let median = *medians(&large).last().unwrap();
        large.push(median + 1 + if height % 13 == 0 { 2_000_000 } else { 0 });
    }
    for span in [3 * 86_400 + 1, 7 * 86_400 + 1, 30 * 86_400 + 1, 365 * 86_400, u32::MAX] {
        assert_window(&large, 0, span);
    }
}

#[test]
fn exact_requested_span_divisors() {
    const DAY: u32 = 86_400;
    // threshold days, divisor at/below it, divisor one second above it.
    for (days, before, after) in [
        (1, 1, 1), (3, 1, 300), (7, 300, 1_800),
        (30, 1_800, 7_200), (90, 7_200, 10_800),
        (180, 10_800, 28_800), (365, 28_800, 28_800),
        (730, 28_800, 43_200), (1095, 43_200, 43_200),
        (1460, 43_200, 86_400),
    ] {
        let threshold = days * DAY;
        for (span, expected) in [(threshold - 1, before), (threshold, before), (threshold + 1, after)] {
            assert_eq!(span_div(range(0, span).span()), expected);
            assert_eq!(span_div(range(u32::MAX - span, u32::MAX).span()), expected);
        }
    }
    assert_eq!(span_div(0), 1);
    assert_eq!(span_div(u32::MAX), 86_400);
    assert!(TimestampRange::new(Timestamp::new(2), Timestamp::new(1)).is_none());
}

#[test]
fn shared_indexed_crossings() {
    let source = [0u32, 1, 1, 1, 4, 9];
    for len in 0..=source.len() {
        for target in 0..=10 {
            let lower = indexed_bound(len, target, IndexedBound::AtLeast, |h| Ok(source[h])).unwrap();
            let upper = indexed_bound(len, target, IndexedBound::Above, |h| Ok(source[h])).unwrap();
            assert_eq!(lower, source[..len].iter().position(|&v| v >= target).unwrap_or(len));
            assert_eq!(upper, source[..len].iter().position(|&v| v > target).unwrap_or(len));
        }
    }
}
