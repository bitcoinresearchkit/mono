#![cfg(feature = "diagnostics")]

use bitview_collections::Windows;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_types::{Height, StoredF32, StoredU64, Timestamp, Version};
use common::init_cache;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableVec, VecIndex, WritableVec, diagnostics};

#[allow(dead_code)]
mod common;

#[test]
fn rolling_resolutions_share_the_cumulative_cache_without_caching_derivations() {
    init_cache();
    const N: usize = 32_768;
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    let days: Vec<_> = (0..N).step_by(144).collect();
    indexes.first_height.day1 =
        common::first_heights("days", days.iter().copied().map(Height::from));
    let timestamps = common::stored::<Height, _>(
        &db,
        "timestamps",
        (0..N).map(|i| Timestamp::from((i * 600) as u32)),
    );
    let starts = Windows {
        _24h: 1,
        _1w: 7,
        _1m: 30,
        _1y: 365,
    }
    .map_with_suffix(|suffix, &days| {
        LazyWindowStartVec::days(suffix, Version::ONE, days, &timestamps)
    });
    let starts_ref = Windows {
        _24h: &starts._24h,
        _1w: &starts._1w,
        _1m: &starts._1m,
        _1y: &starts._1y,
    };
    let mut metric = PerBlockCumulativeRolling::<StoredU64>::forced_import(
        &db,
        "metric",
        Version::ONE,
        &indexes,
        &starts_ref,
    )
    .unwrap();
    let mut total = 0u64;
    let truth: Vec<_> = (0..N)
        .map(|i| {
            let value = (i % 17 + 1) as u64;
            total += value;
            metric.push_block(StoredU64::from(value));
            total
        })
        .collect();
    metric.cumulative.height.write().unwrap();

    let ends: Vec<_> = days
        .iter()
        .enumerate()
        .map(|(day, _)| days.get(day + 1).copied().unwrap_or(N) - 1)
        .collect();
    init_cache().clear();
    // Eviction includes metadata; exclude it from source-value decompression counts.
    indexes.first_height.day1.collect();
    timestamps.collect();
    diagnostics::take();
    for (slot, start) in starts.as_array().into_iter().enumerate() {
        let mapping = start.collect();
        let expected: Vec<_> = ends
            .iter()
            .map(|&end| {
                let start = mapping[end].to_usize();
                Some(StoredU64::from(
                    truth[end] - start.checked_sub(1).map_or(0, |i| truth[i]),
                ))
            })
            .collect();
        assert_eq!(
            metric.sum.as_array()[slot].resolutions.day1.collect(),
            expected
        );
        let decoded = diagnostics::take();
        if slot == 0 {
            assert_eq!(decoded, N / 1024);
        } else {
            assert!(
                decoded <= 1,
                "only a previously unrequested tail page may be decoded"
            );
        }
        assert_eq!(
            metric.sum.as_array()[slot].resolutions.day1.collect(),
            expected
        );
        assert_eq!(
            diagnostics::take(),
            0,
            "repeating a view reuses all its source ranges"
        );
        let averages: Vec<_> = ends
            .iter()
            .zip(&expected)
            .map(|(&end, sum)| {
                Some(StoredF32::from(
                    f64::from(sum.unwrap()) / (end - mapping[end].to_usize() + 1) as f64,
                ))
            })
            .collect();
        assert_eq!(
            metric.average.as_array()[slot].resolutions.day1.collect(),
            averages
        );
        assert_eq!(diagnostics::take(), 0);
    }
    assert_eq!(
        metric.cumulative.day1.collect(),
        ends.iter()
            .map(|&i| Some(StoredU64::from(truth[i])))
            .collect::<Vec<_>>()
    );
    assert_eq!(diagnostics::take(), 0);
    // This block range adds values not requested by the sparse daily views.
    assert_eq!(
        metric.block.collect_range_at(1020, 1030),
        (1020..1030)
            .map(|i| StoredU64::from((i % 17 + 1) as u64))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        diagnostics::take(),
        2,
        "the new range crosses two physical pages"
    );
    metric.block.collect_range_at(1020, 1030);
    assert_eq!(
        diagnostics::take(),
        0,
        "the block view reuses its added range"
    );

    // A same-length replacement invalidates only the changed source suffix.
    metric
        .cumulative
        .height
        .truncate_if_needed(Height::from(N - 1))
        .unwrap();
    metric
        .cumulative
        .height
        .push(StoredU64::from(truth[N - 1] + 123));
    metric.cumulative.height.write().unwrap();
    diagnostics::take(); // Exclude page reads needed to rewrite the compressed tail.
    let values = metric.sum._24h.resolutions.day1.collect();
    let start = starts._24h.collect()[N - 1].to_usize();
    assert_eq!(
        values.last().copied().flatten(),
        Some(StoredU64::from(truth[N - 1] + 123 - truth[start - 1]))
    );
    assert_eq!(
        diagnostics::take(),
        1,
        "only the rewritten tail needs decoding"
    );
    assert_eq!(
        metric.cumulative.day1.collect().last().copied().flatten(),
        Some(StoredU64::from(truth[N - 1] + 123))
    );
    assert_eq!(diagnostics::take(), 0);

    // The lazy cumulative constructor must share its root between its height
    // transform, cumulative resolutions, and rolling views as well.
    let lazy = LazyPerBlockCumulativeRolling::from_cumulative_source(
        "lazy",
        Version::ONE,
        &metric.cumulative.height,
        &starts_ref,
        &indexes,
    );
    init_cache().clear();
    indexes.first_height.day1.collect();
    timestamps.collect();
    diagnostics::take();
    assert_eq!(lazy.sum._24h.resolutions.day1.collect(), values);
    assert_eq!(diagnostics::take(), N / 1024);
    lazy.average._1w.resolutions.day1.collect();
    assert!(
        diagnostics::take() <= 1,
        "additional windows only decode missing tail values"
    );
    lazy.cumulative.resolutions.day1.collect();
    assert_eq!(diagnostics::take(), 0);
    lazy.block.collect_range_at(1020, 1030);
    assert_eq!(diagnostics::take(), 2);
    lazy.block.collect_range_at(1020, 1030);
    assert_eq!(diagnostics::take(), 0);
}
