use std::{fs, path::Path};

use lsm_tree::{
    CompressionType, Config, Error, Result, Slice, Tree,
    config::{
        BlockSizePolicy, CompressionPolicy, FilterPolicy, PartitioningPolicy, RestartIntervalPolicy,
    },
};
use tempfile::tempdir;

fn open(path: &Path) -> Result<Tree> {
    Tree::open(Config::new(path))
}

fn collect(iter: impl Iterator<Item = Result<(Slice, Slice)>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    iter.map(|item| {
        let (key, value) = item?;
        Ok((key.to_vec(), value.to_vec()))
    })
    .collect()
}

#[test]
fn ingestion_reads_and_recovery() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;

    let mut ingestion = tree.ingestion()?;
    ingestion.write("a", "1")?;
    ingestion.write("b", "2")?;
    ingestion.write("c", "3")?;
    ingestion.finish()?;

    let mut ingestion = tree.ingestion()?;
    ingestion.write("b", "20")?;
    ingestion.write_weak_tombstone("c")?;
    ingestion.write("d", "4")?;
    ingestion.finish()?;

    assert_eq!(tree.get("a")?.as_deref(), Some(b"1".as_slice()));
    assert_eq!(tree.get("b")?.as_deref(), Some(b"20".as_slice()));
    assert_eq!(tree.get("c")?, None);
    assert_eq!(tree.get("d")?.as_deref(), Some(b"4".as_slice()));
    assert_eq!(
        collect(tree.iter())?,
        [
            (b"a".to_vec(), b"1".to_vec()),
            (b"b".to_vec(), b"20".to_vec()),
            (b"d".to_vec(), b"4".to_vec())
        ],
    );
    assert_eq!(
        collect(tree.range("b"..="d"))?,
        [
            (b"b".to_vec(), b"20".to_vec()),
            (b"d".to_vec(), b"4".to_vec())
        ],
    );

    drop(tree);
    let tree = open(directory.path())?;
    assert_eq!(tree.get("b")?.as_deref(), Some(b"20".as_slice()));
    assert_eq!(tree.get("c")?, None);

    let mut ingestion = tree.ingestion()?;
    ingestion.write("e", "5")?;
    ingestion.finish()?;
    drop(tree);

    assert_eq!(
        open(directory.path())?.get("e")?.as_deref(),
        Some(b"5".as_slice())
    );
    Ok(())
}

#[test]
fn transitive_overlap_preserves_newest_value_after_recovery() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;

    let mut ingestion = tree.ingestion()?;
    ingestion.write("a", "oldest")?;
    ingestion.write("c", "oldest")?;
    ingestion.finish()?;

    let mut ingestion = tree.ingestion()?;
    ingestion.write("a", "middle")?;
    ingestion.write("n", "middle")?;
    ingestion.write("z", "middle")?;
    ingestion.finish()?;

    let mut ingestion = tree.ingestion()?;
    ingestion.write("m", "newest")?;
    ingestion.write("n", "newest")?;
    ingestion.write("p", "newest")?;
    ingestion.finish()?;

    assert_eq!(tree.get("n")?.as_deref(), Some(b"newest".as_slice()));
    drop(tree);
    assert_eq!(
        open(directory.path())?.get("n")?.as_deref(),
        Some(b"newest".as_slice())
    );
    Ok(())
}

#[test]
fn prefix_and_double_ended_ranges() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    let mut ingestion = tree.ingestion()?;
    ingestion.write("addr/1", "a")?;
    ingestion.write("addr/2", "b")?;
    ingestion.write("block/1", "c")?;
    ingestion.finish()?;

    assert_eq!(
        collect(tree.prefix("addr/"))?,
        [
            (b"addr/1".to_vec(), b"a".to_vec()),
            (b"addr/2".to_vec(), b"b".to_vec())
        ],
    );

    let mut range = tree.range("addr/1".."block/2");
    assert_eq!(
        range.next().transpose()?.map(|item| item.0),
        Some(Slice::from("addr/1"))
    );
    assert_eq!(
        range.next_back().transpose()?.map(|item| item.0),
        Some(Slice::from("block/1"))
    );
    assert_eq!(
        range.next().transpose()?.map(|item| item.0),
        Some(Slice::from("addr/2"))
    );
    assert!(range.next().is_none());
    Ok(())
}

#[test]
fn compaction_preserves_latest_values_and_open_readers() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;

    for generation in 0..8_u8 {
        let mut ingestion = tree.ingestion()?;
        for key in 0..64_u8 {
            if generation == 7 && key == 10 {
                ingestion.write_weak_tombstone([key])?;
            } else {
                ingestion.write([key], [generation])?;
            }
        }
        ingestion.finish()?;
    }

    let reader = tree.iter();
    tree.compact()?;
    let stable_version = tree.current_version_id();
    tree.compact()?;
    assert_eq!(tree.current_version_id(), stable_version);

    assert_eq!(
        collect(reader)
            .unwrap_or_else(|error| panic!("open reader failed: {error:?}"))
            .len(),
        63,
    );
    for key in 0..64_u8 {
        let expected = (key != 10).then_some([7_u8]);
        assert_eq!(
            tree.get([key])?.as_deref(),
            expected.as_ref().map(|value| value.as_slice())
        );
    }

    drop(tree);
    let tree = open(directory.path())?;
    assert_eq!(collect(tree.iter())?.len(), 63);
    for key in 0..64_u8 {
        let expected = (key != 10).then_some([7_u8]);
        assert_eq!(
            tree.get([key])?.as_deref(),
            expected.as_ref().map(|value| value.as_slice())
        );
    }
    Ok(())
}

#[test]
fn compaction_bounds_overlapping_l0_runs() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;

    for generation in 0..4_u8 {
        let mut ingestion = tree.ingestion()?;
        for key in 0..64_u8 {
            ingestion.write([key], [generation])?;
        }
        ingestion.finish()?;
    }

    assert_eq!(4, tree.l0_run_count());
    let version = tree.current_version_id();
    tree.compact()?;

    assert!(tree.current_version_id() > version);
    assert!(tree.l0_run_count() < 4);
    for key in 0..64_u8 {
        assert_eq!(tree.get([key])?.as_deref(), Some([3_u8].as_slice()));
    }
    Ok(())
}

#[test]
fn empty_ingestion_does_not_publish() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    let version = tree.current_version_id();
    tree.ingestion()?.finish()?;
    assert_eq!(tree.current_version_id(), version);
    Ok(())
}

#[test]
fn seeks_across_restarts_and_inline_boundaries() -> Result<()> {
    for compression in [CompressionType::None, CompressionType::Lz4] {
        for key_len in [10, 12, 13, 20, 32] {
            let directory = tempdir()?;
            let config = Config::new(directory.path())
                .data_block_size_policy(BlockSizePolicy::all(512))
                .data_block_restart_interval_policy(RestartIntervalPolicy::all(8))
                .data_block_compression_policy(CompressionPolicy::all(compression));
            let tree = config.open()?;
            let key = |i: u64| {
                let mut key = vec![17; key_len];
                key[..8].copy_from_slice(&i.to_be_bytes());
                key
            };
            let expected: Vec<_> = (0..192_u64)
                .map(|i| {
                    (
                        key(i * 2),
                        vec![i as u8; [0, 4, 12, 13, 64][i as usize % 5]],
                    )
                })
                .collect();
            let mut ingestion = tree.ingestion()?;
            for (key, value) in &expected {
                ingestion.write(key.as_slice(), value.as_slice())?;
            }
            ingestion.finish()?;
            drop(tree);
            let tree = open(directory.path())?;

            for (i, (stored_key, value)) in expected.iter().enumerate() {
                assert_eq!(tree.get(stored_key)?.as_deref(), Some(value.as_slice()));
                assert!(tree.get(key(i as u64 * 2 + 1))?.is_none());
            }
            assert_eq!(collect(tree.iter())?, expected);
            for start in (0..192).step_by(7) {
                let end = (start + 17).min(191);
                let range = key(start * 2 + 1)..=key(end * 2);
                let wanted = &expected[start as usize + 1..=end as usize];
                assert_eq!(collect(tree.range(range.clone()))?, wanted);
                assert_eq!(
                    collect(tree.range(range).rev())?,
                    wanted.iter().rev().cloned().collect::<Vec<_>>()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn recovery_rejects_a_truncated_manifest() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    let mut ingestion = tree.ingestion()?;
    ingestion.write("a", "1")?;
    ingestion.finish()?;
    drop(tree);

    let path = directory.path().join("current");
    let mut bytes = fs::read(&path)?;
    bytes.truncate(5);
    fs::write(path, bytes)?;

    assert!(matches!(open(directory.path()), Err(Error::Unrecoverable)));
    Ok(())
}

#[test]
fn recovery_rejects_old_manifest_versions_without_rewriting() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    drop(tree);

    let path = directory.path().join("current");
    let mut bytes = fs::read(&path)?;
    for version in [8, 9, 10] {
        *bytes.get_mut(3).ok_or(Error::Unrecoverable)? = version;
        fs::write(&path, &bytes)?;
        assert!(matches!(
            open(directory.path()),
            Err(Error::InvalidVersion(found)) if found == version
        ));
        assert_eq!(fs::read(&path)?, bytes);
    }
    Ok(())
}

#[test]
#[should_panic(expected = "ingestion keys must be strictly increasing")]
fn ingestion_rejects_unsorted_keys_in_debug_builds() {
    let directory = tempdir().expect("temporary directory");
    let tree = open(directory.path()).expect("tree");
    let mut ingestion = tree.ingestion().expect("ingestion");
    ingestion.write("b", "1").expect("write");
    ingestion.write("a", "2").expect("write");
}

#[test]
fn fixed_records_preserve_values_tombstones_and_order() -> Result<()> {
    fn check<const K: usize, const V: usize>(
        compression: CompressionType,
        fixed_compaction: bool,
    ) -> Result<()> {
        let dir = tempdir()?;
        let config = Config::new(dir.path())
            .data_block_compression_policy(CompressionPolicy::all(compression))
            .filter_block_partitioning_policy(PartitioningPolicy::all(true))
            .index_block_partitioning_policy(PartitioningPolicy::all(true));
        let key = |i: u32| {
            let mut bytes = [0xff; K];
            bytes[..4].copy_from_slice(&i.to_be_bytes());
            bytes
        };
        let value = |i: u32, generation: u8| [i.wrapping_add(u32::from(generation)) as u8; V];
        let expected: Vec<_> = (0..1024)
            .filter(|i| i % 6 != 0)
            .map(|i| (key(i), value(i, u8::from(i % 3 == 0))))
            .collect();
        let tree = config.open()?;
        let mut ingest = tree.ingestion_as::<[u8; K], [u8; V]>()?;
        for i in 0..1024 {
            ingest.write(key(i), value(i, 0))?;
        }
        ingest.finish()?;
        for _ in 0..3 {
            let mut ingest = tree.ingestion_as::<[u8; K], [u8; V]>()?;
            for i in (0..1024).step_by(3) {
                if i % 6 == 0 {
                    ingest.write_weak_tombstone(key(i))?;
                } else {
                    ingest.write(key(i), value(i, 1))?;
                }
            }
            ingest.finish()?;
        }

        for phase in 0..3 {
            if phase == 1 {
                let version = tree.current_version_id();
                if fixed_compaction {
                    tree.compact_as::<[u8; K], [u8; V]>()?;
                } else {
                    tree.compact()?;
                }
                assert!(tree.current_version_id() > version);
                assert_eq!(tree.l0_run_count(), 0);
            }
            let reopened;
            let current = if phase == 2 {
                reopened = Config::new(dir.path()).open()?;
                &reopened
            } else {
                &tree
            };
            let forward = current
                .iter_as::<[u8; K], [u8; V]>()
                .collect::<Result<Vec<_>>>()?;
            let reverse = current
                .iter_as::<[u8; K], [u8; V]>()
                .rev()
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(forward, expected);
            assert_eq!(reverse, expected.iter().copied().rev().collect::<Vec<_>>());
            let mut mixed = current.iter_as::<[u8; K], [u8; V]>();
            let mut reference = expected.iter();
            while let Some(item) = reference.next() {
                assert_eq!(mixed.next().transpose()?, Some(*item));
                assert_eq!(
                    mixed.next_back().transpose()?,
                    reference.next_back().copied()
                );
            }
            assert!(mixed.next().is_none());
            assert!(mixed.next_back().is_none());
            for i in 0..1024 {
                assert_eq!(
                    current.get_as::<[u8; V]>(&key(i))?,
                    (i % 6 != 0).then(|| value(i, u8::from(i % 3 == 0)))
                );
            }
            let ranged = current
                .range_as::<[u8; K], [u8; V], _, _>(key(101)..key(611))
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(
                ranged,
                expected
                    .iter()
                    .copied()
                    .filter(|(k, _)| *k >= key(101) && *k < key(611))
                    .collect::<Vec<_>>()
            );
            let prefixed = current
                .prefix_as::<[u8; K], [u8; V]>(&[0, 0, 0])
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(
                prefixed,
                expected
                    .iter()
                    .copied()
                    .filter(|(k, _)| k.starts_with(&[0, 0, 0]))
                    .collect::<Vec<_>>()
            );
            assert!(matches!(current.get_as::<[u8; 1]>(&key(1)),
                Err(Error::InvalidRecordLength { expected: 1, actual }) if actual == V));
            assert!(matches!(current.iter_as::<[u8; 1], [u8; V]>().next(),
                Some(Err(Error::InvalidRecordLength { expected: 1, actual })) if actual == K));
        }
        Ok(())
    }
    for compression in [CompressionType::None, CompressionType::Lz4] {
        for fixed_compaction in [false, true] {
            check::<8, 4>(compression, fixed_compaction)?;
            check::<8, 0>(compression, fixed_compaction)?;
            check::<10, 0>(compression, fixed_compaction)?;
        }
    }
    Ok(())
}

#[test]
fn compaction_width_mismatch_preserves_inputs_and_allows_retry() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    for generation in 0..4_u32 {
        let mut ingest = tree.ingestion()?;
        for key in 0..1024_u64 {
            if key == 1023 {
                ingest.write(key.to_be_bytes(), [7; 3])?;
            } else {
                ingest.write(key.to_be_bytes(), generation.to_be_bytes())?;
            }
        }
        ingest.finish()?;
    }
    let version = tree.current_version_id();
    let expected = collect(tree.iter())?;
    assert!(matches!(
        tree.compact_as::<[u8; 7], [u8; 4]>(),
        Err(Error::InvalidRecordLength {
            expected: 7,
            actual: 8
        })
    ));
    for _ in 0..2 {
        assert!(matches!(
            tree.compact_as::<[u8; 8], [u8; 4]>(),
            Err(Error::InvalidRecordLength {
                expected: 4,
                actual: 3
            })
        ));
        assert_eq!(tree.current_version_id(), version);
        assert_eq!(tree.l0_run_count(), 4);
        assert_eq!(collect(tree.iter())?, expected);
    }
    tree.compact()?;
    assert_eq!(tree.l0_run_count(), 0);
    assert_eq!(collect(tree.iter())?, expected);
    drop(tree);
    assert_eq!(collect(open(directory.path())?.iter())?, expected);
    Ok(())
}

#[test]
fn fixed_key_only_records_with_disabled_partitioned_filters() -> Result<()> {
    let dir = tempdir()?;
    let tree = Config::new(dir.path())
        .filter_policy(FilterPolicy::disabled())
        .filter_block_partitioning_policy(PartitioningPolicy::all(true))
        .open()?;
    let mut ingest = tree.ingestion_as::<[u8; 8], [u8; 0]>()?;
    ingest.write(1_u64.to_be_bytes(), [])?;
    ingest.finish()?;
    assert_eq!(tree.get_as::<[u8; 0]>(&1_u64.to_be_bytes())?, Some([]));
    assert_eq!(tree.get_as::<[u8; 0]>(&2_u64.to_be_bytes())?, None);
    Ok(())
}

#[test]
fn single_run_ranges_filter_tombstones_and_preserve_both_directions() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    let key = |group: u32, row: u32| {
        let mut key = [0; 8];
        key[..4].copy_from_slice(&group.to_be_bytes());
        key[4..].copy_from_slice(&row.to_be_bytes());
        key
    };
    assert!(tree.iter_as::<[u8; 8], [u8; 0]>().next().is_none());
    for group in 0..4 {
        let mut ingestion = tree.ingestion_as::<[u8; 8], [u8; 0]>()?;
        for row in 0..32 {
            if row % 7 == 0 {
                ingestion.write_weak_tombstone(key(group, row))?;
            } else {
                ingestion.write(key(group, row), [])?;
            }
        }
        ingestion.finish()?;
    }
    for phase in 0..2 {
        if phase == 1 {
            tree.compact()?;
        }
        for group in 0..4 {
            let expected: Vec<_> = (3..23)
                .filter(|row| row % 7 != 0)
                .map(|row| (key(group, row), []))
                .collect();
            let range = key(group, 3)..key(group, 23);
            let scan = || tree.range_as::<[u8; 8], [u8; 0], _, _>(range.clone());
            assert_eq!(scan().collect::<Result<Vec<_>>>()?, expected);
            assert_eq!(
                scan().rev().collect::<Result<Vec<_>>>()?,
                expected.iter().copied().rev().collect::<Vec<_>>()
            );
            let mut mixed = scan();
            let mut reference = expected.iter();
            while let Some(front) = reference.next() {
                assert_eq!(mixed.next().transpose()?, Some(*front));
                assert_eq!(
                    mixed.next_back().transpose()?,
                    reference.next_back().copied()
                );
            }
            assert!(mixed.next().is_none());
            assert!(mixed.next_back().is_none());
        }
        assert!(
            tree.prefix_as::<[u8; 8], [u8; 0]>(&9_u32.to_be_bytes())
                .next()
                .is_none()
        );
    }
    Ok(())
}
