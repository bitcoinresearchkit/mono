use bitview_vecs::{LazyRollingDistribution, PerBlockDistribution, RollingDistribution};
use brk_exit::Exit;
use brk_types::{Height, StoredU64, TxIndex, VSize, Version, get_percentile};
use common::{indexes, init_cache, stored};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Database, Ident, ImportableVec, PcoVec, ReadableVec, WritableVec,
};

mod common;

fn weighted_reference(values: &[(StoredU64, VSize)], rank: f64) -> StoredU64 {
    let total: u64 = values.iter().map(|(_, w)| u64::from(*w)).sum();
    let target = (total as f64 * rank).round() as u64;
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
fn stored_distributions_match_reference_after_resume_rewind_and_version_reset() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = indexes(&db);
    let blocks: Vec<Vec<_>> = (0..24u64)
        .map(|height| {
            (0..height % 11)
                .map(|tx| {
                    (
                        StoredU64::from((height * 7 + tx * 3) % 13),
                        VSize::new((tx * 19 + height) % 100),
                    )
                })
                .collect()
        })
        .collect();
    let mut values =
        PcoVec::<TxIndex, StoredU64>::forced_import(&db, "values", Version::ONE).unwrap();
    let mut weights =
        PcoVec::<TxIndex, VSize>::forced_import(&db, "weights", Version::ONE).unwrap();
    for &(value, weight) in blocks.iter().flatten() {
        values.push(value);
        weights.push(weight);
    }
    values.write().unwrap();
    weights.write().unwrap();
    let counts = stored::<Height, _>(
        &db,
        "counts",
        blocks.iter().map(|b| StoredU64::from(b.len())),
    );
    let mut offset = 0usize;
    let first = stored::<Height, _>(
        &db,
        "first_tx",
        blocks.iter().map(|b| {
            let start = TxIndex::from(offset);
            offset += b.len();
            start
        }),
    );
    let exit = Exit::new();
    for weighted in [false, true] {
        for nblocks in [1, 6] {
            for skip in [0, 1, 20] {
                let name = format!("output_{weighted}_{nblocks}_{skip}");
                let mut output =
                    PerBlockDistribution::forced_import(&db, &name, Version::ONE, &indexes)
                        .unwrap();
                let compute = |output: &mut PerBlockDistribution<StoredU64>, from: usize| {
                    let from = Height::from(from);
                    match (weighted, nblocks) {
                        (false, 1) => {
                            output.compute_with_skip(from, &values, &first, &counts, &exit, skip)
                        }
                        (true, 1) => output.compute_with_skip_weighted(
                            from, &values, &weights, &first, &counts, &exit, skip,
                        ),
                        (false, _) => output.compute_from_nblocks(
                            from, &values, &first, &counts, nblocks, &exit, skip,
                        ),
                        (true, _) => output.compute_from_nblocks_weighted(
                            from, &values, &weights, &first, &counts, nblocks, &exit, skip,
                        ),
                    }
                    .unwrap();
                };
                let expected: Vec<[StoredU64; 7]> = (0..blocks.len())
                    .map(|height| {
                        let mut population: Vec<_> = blocks
                            [height.saturating_sub(nblocks - 1)..=height]
                            .iter()
                            .flat_map(|b| b.iter().skip(skip).copied())
                            .filter(|(v, _)| skip == 0 || u64::from(*v) > 0)
                            .collect();
                        population.sort_unstable();
                        if population.is_empty() {
                            return [StoredU64::from(0u64); 7];
                        }
                        let unweighted: Vec<_> = population.iter().map(|&(v, _)| v).collect();
                        let rank = |p| {
                            if weighted {
                                weighted_reference(&population, p)
                            } else {
                                get_percentile(&unweighted, p)
                            }
                        };
                        [
                            population[0].0,
                            population.last().unwrap().0,
                            rank(0.1),
                            rank(0.25),
                            rank(0.5),
                            rank(0.75),
                            rank(0.9),
                        ]
                    })
                    .collect();
                for phase in 0..5 {
                    match phase {
                        2 => {
                            output.median.height.truncate_if_needed_at(13).unwrap();
                        }
                        4 => {
                            output
                                .pct25
                                .height
                                .validate_computed_version_or_reset(Version::ZERO)
                                .unwrap();
                        }
                        _ => {}
                    }
                    compute(&mut output, if phase == 3 { 7 } else { blocks.len() });
                    for (metric, vec) in [
                        &output.min.height,
                        &output.max.height,
                        &output.pct10.height,
                        &output.pct25.height,
                        &output.median.height,
                        &output.pct75.height,
                        &output.pct90.height,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        assert_eq!(
                            vec.collect_range_at(0, vec.len()),
                            expected
                                .iter()
                                .map(|values| values[metric])
                                .collect::<Vec<_>>(),
                            "{name} phase={phase} metric={metric}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn lazy_rolling_distribution_preserves_all_stat_window_mappings() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = indexes(&db);
    let mut source =
        RollingDistribution::<StoredU64>::forced_import(&db, "source", Version::ONE, &indexes)
            .unwrap();
    let mut value = 0u64;
    let stats = &mut source.0;
    for windows in [
        &mut stats.min,
        &mut stats.max,
        &mut stats.pct10,
        &mut stats.pct25,
        &mut stats.median,
        &mut stats.pct75,
        &mut stats.pct90,
    ] {
        for window in windows.0.as_mut_array() {
            value += 1;
            window.height.push(StoredU64::from(value));
            window.height.write().unwrap();
        }
    }
    let lazy = LazyRollingDistribution::<StoredU64, StoredU64>::from_rolling_distribution::<Ident>(
        "converted",
        Version::ONE,
        &source,
    );
    for (stat_index, (suffix, windows)) in [
        ("min", &lazy.min),
        ("max", &lazy.max),
        ("pct10", &lazy.pct10),
        ("pct25", &lazy.pct25),
        ("median", &lazy.median),
        ("pct75", &lazy.pct75),
        ("pct90", &lazy.pct90),
    ]
    .into_iter()
    .enumerate()
    {
        for (window_index, (window_suffix, window)) in ["24h", "1w", "1m", "1y"]
            .into_iter()
            .zip(windows.as_array())
            .enumerate()
        {
            assert_eq!(
                window.height.name(),
                format!("converted_{suffix}_{window_suffix}")
            );
            assert_eq!(
                window.height.collect_one_at(0),
                Some(StoredU64::from((stat_index * 4 + window_index + 1) as u64))
            );
        }
    }
}
