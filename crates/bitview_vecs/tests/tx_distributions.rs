use bitview_vecs::{LazyPreviousDeltaVec, TxDerivedDistribution};
use brk_exit::Exit;
use brk_types::{Height, Lengths, StoredU64, TxIndex, VSize, Version, get_percentile};
use common::{indexes, init_cache, stored};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableVec, WritableVec};

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
                        VSize::new(60 + (tx * 19 + height) % 100),
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
                            *weight = VSize::new(60 + (u64::from(*weight) + 17) % 100);
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
        }
    }
}
