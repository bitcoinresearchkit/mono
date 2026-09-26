use bitview_traversable::{Traversable, TreeNode};
use bitview_vecs::{
    LazyPreviousDeltaVec, LazyRollingDistribution, LazyTxDerivedDistribution, RollingDistribution,
    TxDerivedDistribution,
};
use brk_exit::Exit;
use brk_types::{Height, Lengths, StoredU64, TxIndex, VSize, Version, get_percentile};
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
    let mut blocks: Vec<Vec<_>> = (0..24u64)
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
    let mut total = 0usize;
    let cumulative = stored::<Height, _>(
        &db,
        "cumulative",
        blocks.iter().map(|b| {
            total += b.len();
            StoredU64::from(total)
        }),
    );
    let mut indexes = indexes;
    indexes.height_tx_index_count = LazyPreviousDeltaVec::new("counts", Version::ONE, &cumulative);
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
        for skip in [0, 1, 20] {
            let name = format!("output_{weighted}_{skip}");
            let mut output =
                TxDerivedDistribution::forced_import(&db, &name, Version::ONE, &indexes).unwrap();
            for phase in 0..7 {
                match phase {
                    2 => output
                        .block
                        .median
                        .height
                        .truncate_if_needed_at(13)
                        .unwrap(),
                    3 => {
                        // Replace a suffix in place, as a reorg does, without changing source versions.
                        let start_tx = usize::from(first.collect_one_at(7).unwrap());
                        values.truncate_if_needed_at(start_tx).unwrap();
                        weights.truncate_if_needed_at(start_tx).unwrap();
                        for (value, weight) in blocks[7..].iter_mut().flatten() {
                            *value = StoredU64::from((u64::from(*value) + 5) % 13);
                            *weight = VSize::new((u64::from(*weight) + 17) % 100);
                            values.push(*value);
                            weights.push(*weight);
                        }
                        values.write().unwrap();
                        weights.write().unwrap();
                    }
                    4 => output
                        ._6b
                        .pct25
                        .height
                        .validate_computed_version_or_reset(Version::ZERO)
                        .unwrap(),
                    5 => output._6b.max.height.truncate_if_needed_at(9).unwrap(),
                    6 => {
                        drop(output);
                        output = TxDerivedDistribution::forced_import(
                            &db,
                            &name,
                            Version::ONE,
                            &indexes,
                        )
                        .unwrap();
                    }
                    _ => {}
                }
                let lengths = Lengths {
                    height: Height::from(if phase == 3 { 7 } else { blocks.len() }),
                    ..Lengths::default()
                };
                if weighted {
                    output.derive_from_with_skip_weighted(
                        &indexes, &lengths, &first, &values, &weights, &exit, skip,
                    )
                } else {
                    output.derive_from_with_skip(&indexes, &lengths, &first, &values, &exit, skip)
                }
                .unwrap();
                for (nblocks, output) in [(1, &output.block), (6, &output._6b)] {
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
                            "{name} phase={phase} window={nblocks} metric={metric}"
                        );
                    }
                }
            }
            let lazy = LazyTxDerivedDistribution::<StoredU64, StoredU64>::from_tx_derived::<Ident>(
                "converted",
                Version::ONE,
                &output,
            );
            // Storage names and public catalog paths do not depend on the removed wrappers.
            for tree in [output.to_tree_node(), lazy.to_tree_node()] {
                let TreeNode::Branch(children) = tree else {
                    panic!("expected distribution branch")
                };
                assert_eq!(
                    children.keys().map(String::as_str).collect::<Vec<_>>(),
                    ["block", "6b"]
                );
            }
            assert_eq!(output._6b.median.height.name(), format!("{name}_6b_median"));
            assert_eq!(lazy._6b.median.name(), "converted_6b_median");
            assert_eq!(
                lazy._6b.median.collect(),
                output._6b.median.height.collect()
            );
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
