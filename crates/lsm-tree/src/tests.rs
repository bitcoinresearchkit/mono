use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{ErrorKind, Write},
    ops::{Bound, RangeBounds},
    path::Path,
};

use byteorder::{LittleEndian, WriteBytesExt};
use tempfile::{NamedTempFile, tempdir};
use xxhash_rust::xxh3;

use crate::{
    CompressionType, Config, Error, InternalValue, KeyRange, Result, Slice, Tree, ValueType,
    coding::{Decode, Encode},
    compaction::stream::CompactionStream,
    config::{FilterPolicy, PartitioningPolicy, PinningPolicy, RestartIntervalPolicy},
    file::{CURRENT_VERSION_FILE, MAGIC_BYTES, persist_temp_file, read_exact, rewrite_atomic},
    key::InternalKey,
    mvcc_stream::MvccStream,
    range::prefix_to_range,
    table::{
        BlockHandle, BlockOffset, DataBlock,
        block::{
            Block, BlockType, Header, Lz4Scratch, ParsedItem, TRAILER_START_MARKER,
            binary_index::Reader as BinaryIndexReader,
        },
        filter::{
            BloomConstructionPolicy,
            standard_bloom::{Builder as BloomBuilder, StandardBloomFilterReader},
        },
    },
    version::{
        DEFAULT_LEVEL_COUNT, Level, Version,
        optimize::optimize_runs,
        recovery::Recovery,
        run::{Ranged, Run},
    },
};

/// Deterministic xorshift generator for generated cases.
fn generator(mut state: u64) -> impl FnMut(u64) -> u64 {
    move |bound| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    }
}

fn collect(iter: impl Iterator<Item = Result<(Slice, Slice)>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    iter.map(|item| {
        let (key, value) = item?;
        Ok((key.to_vec(), value.to_vec()))
    })
    .collect()
}

fn open(path: &Path) -> Result<Tree> {
    Tree::open(Config::new(path))
}

// Codec

#[test]
fn block_codec_roundtrips_and_rejects_corrupt_lengths() -> Result<()> {
    let header = Header {
        block_type: BlockType::Data,
        data_length: 252_356,
        uncompressed_length: 124_124_124,
    };
    let mut bytes = header.encode_into_vec();
    assert_eq!(bytes.len(), Header::serialized_len());
    assert_eq!(header, Header::decode_from(&mut &bytes[..])?);
    bytes[0] ^= 1;
    assert!(matches!(
        Header::decode_from(&mut &bytes[..]),
        Err(Error::InvalidHeader("Block"))
    ));

    for compression in [CompressionType::None, CompressionType::Lz4] {
        let bytes = compression.encode_into_vec();
        assert_eq!(bytes.len(), 1);
        assert_eq!(CompressionType::decode_from(&mut &bytes[..])?, compression);
    }
    assert!(ValueType::try_from(TRAILER_START_MARKER).is_err());

    // One scratch serves growing, large (64 KiB hash-table upgrade) and shrinking blocks.
    let mut scratch = Lz4Scratch::default();
    for len in [1_usize, 18, 4_096, 70_000, 100] {
        let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8 ^ (i / 7) as u8).collect();
        let mut plain = Vec::new();
        Block::write_into(&mut plain, &data, BlockType::Data)?;
        let mut lz4 = Vec::new();
        Block::write_lz4_into(&mut lz4, &data, BlockType::Data, &mut scratch)?;
        assert_eq!(
            &*Block::from_reader(&mut &plain[..], CompressionType::None)?.data,
            &data[..]
        );
        assert_eq!(
            &*Block::from_reader(&mut &lz4[..], CompressionType::Lz4)?.data,
            &data[..]
        );
    }

    let directory = tempdir()?;
    let path = directory.path().join("block");
    let mut bytes = Vec::new();
    Block::write_into(&mut bytes, b"abcdef", BlockType::Data)?;
    bytes.push(0);
    fs::write(&path, &bytes)?;
    let handle = BlockHandle::new(BlockOffset(0), bytes.len() as u32);
    assert!(matches!(
        Block::from_file(&File::open(path)?, handle, CompressionType::None),
        Err(Error::InvalidHeader("Block length"))
    ));

    for (compression, uncompressed_length) in
        [(CompressionType::None, 2), (CompressionType::Lz4, 257)]
    {
        let mut bytes = Header {
            block_type: BlockType::Data,
            data_length: 1,
            uncompressed_length,
        }
        .encode_into_vec();
        bytes.push(0);
        assert!(matches!(
            Block::from_reader(&mut &bytes[..], compression),
            Err(Error::InvalidHeader("Block length"))
        ));
    }
    Ok(())
}

#[test]
fn binary_index_search_handles_both_offset_widths() {
    for (offsets, width) in [
        ([0_u32, 17, 31, 100, 65535], 2),
        ([0_u32, 17, 65536, 100_000, u32::MAX], 4),
    ] {
        // Include a prefix to exercise an index at a nonzero block offset.
        let mut bytes = vec![9; 3];
        for offset in offsets {
            bytes.extend_from_slice(&offset.to_le_bytes()[..usize::from(width)]);
        }
        let reader = BinaryIndexReader::new(&bytes, 3, 5, width);
        for needle in [0, 1, 17, 18, 31, 100, 65535, 65536, 100_001, u32::MAX] {
            let expected = offsets
                .iter()
                .take_while(|offset| **offset < needle)
                .count();
            assert_eq!(
                reader.partition_point(|offset| offset < needle as usize),
                expected
            );
        }
        for (index, offset) in offsets.into_iter().enumerate() {
            assert_eq!(reader.get(index), offset as usize);
        }
    }
    for width in [2, 4] {
        assert_eq!(
            BinaryIndexReader::new(&[], 0, 0, width).partition_point(|_| true),
            0
        );
    }
}

fn data_block(bytes: Vec<u8>) -> (DataBlock, Slice) {
    let bytes = Slice::from(bytes);
    let block = DataBlock::new(Block {
        data: bytes.clone(),
        header: Header {
            block_type: BlockType::Data,
            data_length: 0,
            uncompressed_length: 0,
        },
    });
    (block, bytes)
}

fn parts(item: InternalValue) -> (Slice, u64, ValueType, Slice) {
    (
        item.key.user_key,
        item.key.seqno,
        item.key.value_type,
        item.value,
    )
}

#[test]
fn data_blocks_roundtrip_restart_groups_and_fixed_widths() -> Result<()> {
    for interval in [1_u8, 2, 3, 4, 8, 16, 31, 255] {
        let width = u16::from(interval);
        for count in [
            1,
            width.saturating_sub(1).max(1),
            width,
            width + 1,
            2 * width + 1,
        ] {
            let items: Vec<_> = (0..count)
                .map(|i| {
                    let (value, kind) = if i % 3 == 0 {
                        (Slice::empty(), ValueType::Tombstone)
                    } else {
                        (Slice::from(i.to_be_bytes()), ValueType::Value)
                    };
                    InternalValue::from_components(i.to_be_bytes(), value, u64::from(i), kind)
                })
                .collect();
            let mut bytes = Vec::new();
            DataBlock::encode_into(&mut bytes, &items, interval)?;
            let (block, bytes) = data_block(bytes);
            assert_eq!(block.len(), items.len());
            assert_eq!(
                block.binary_index_len(),
                u32::from(count).div_ceil(u32::from(interval))
            );
            let expected: Vec<_> = items.iter().cloned().map(parts).collect();
            let forward: Vec<_> = block
                .iter()
                .map(|item| parts(item.materialize(&bytes)))
                .collect();
            let reverse: Vec<_> = block
                .iter()
                .rev()
                .map(|item| parts(item.materialize(&bytes)))
                .collect();
            assert_eq!(forward, expected);
            assert_eq!(reverse, expected.into_iter().rev().collect::<Vec<_>>());
            for item in items {
                assert_eq!(
                    block.point_read(&item.key.user_key).map(parts),
                    Some(parts(item))
                );
            }
        }
    }

    let items: Vec<_> = (0..256_u64)
        .map(|i| {
            InternalValue::from_components(
                i.to_be_bytes(),
                (i as u32).to_be_bytes(),
                0,
                ValueType::Value,
            )
        })
        .collect();
    let mut fixed = Vec::new();
    DataBlock::encode_into(&mut fixed, &items, 16)?;
    let mut variable = Vec::new();
    DataBlock::encode_into_with_fixed_lengths(
        &mut variable,
        &mut Vec::new(),
        &items,
        16,
        None,
        None,
    )?;
    assert!(
        fixed.len() + items.len() < variable.len(),
        "fixed-width encoding should omit at least one length per item",
    );
    for bytes in [fixed, variable] {
        let (block, bytes) = data_block(bytes);
        let decoded: Vec<_> = block
            .iter()
            .map(|item| parts(item.materialize(&bytes)))
            .collect();
        assert_eq!(
            decoded,
            items.iter().cloned().map(parts).collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[test]
fn bloom_filters_roundtrip_without_false_negatives() -> Result<()> {
    let keys: Vec<_> = (0..1_000_u32).map(u32::to_be_bytes).collect();
    for policy in [
        BloomConstructionPolicy::FalsePositiveRate(0.0001),
        BloomConstructionPolicy::BitsPerKey(10.0),
    ] {
        let mut builder = policy.init(keys.len());
        for key in &keys {
            builder.set_with_hash(BloomBuilder::get_hash(key));
        }
        let mut bytes = builder.build();
        let borrowed = StandardBloomFilterReader::new(bytes.as_slice())?;
        let owned = StandardBloomFilterReader::new(Slice::from(bytes.clone()))?;
        assert_eq!((borrowed.size(), owned.size()), (bytes.len(), bytes.len()));
        for key in &keys {
            let hash = BloomBuilder::get_hash(key);
            assert!(borrowed.contains_hash(hash) && owned.contains_hash(hash));
        }

        bytes.pop();
        assert!(matches!(
            StandardBloomFilterReader::new(bytes),
            Err(Error::InvalidHeader("BloomFilter"))
        ));
    }
    Ok(())
}

// Ordering

#[test]
fn keys_order_by_user_key_then_newest_and_prefixes_carry() {
    use ValueType::Value as V;

    let mut keys = [
        InternalKey::new(b"d", 3, V),
        InternalKey::new(b"a", 0, V),
        InternalKey::new(b"b", 0, V),
        InternalKey::new(b"a", 2, V),
        InternalKey::new(b"b", 1, V),
        InternalKey::new(b"a", 1, V),
        InternalKey::new(b"c", 3, V),
    ];
    keys.sort();
    assert_eq!(
        keys.map(|key| (key.user_key, key.seqno)),
        [
            (Slice::from(b"a"), 2),
            (Slice::from(b"a"), 1),
            (Slice::from(b"a"), 0),
            (Slice::from(b"b"), 1),
            (Slice::from(b"b"), 0),
            (Slice::from(b"c"), 3),
            (Slice::from(b"d"), 3),
        ]
    );

    for (prefix, upper) in [
        (&b"abc"[..], Bound::Excluded(&b"abd"[..])),
        (&[], Bound::Unbounded),
        (&[255, 255, 255], Bound::Unbounded),
        (&[0, 255], Bound::Excluded(&[1][..])),
        (&[0, 2, 255], Bound::Excluded(&[0, 3][..])),
    ] {
        let lower = if prefix.is_empty() {
            Bound::Unbounded
        } else {
            Bound::Included(Slice::from(prefix))
        };
        assert_eq!(prefix_to_range(prefix), (lower, upper.map(Slice::from)));
    }
}

/// Generated multi-version streams: reads see each key's newest version, in either direction.
#[test]
fn mvcc_stream_yields_the_newest_version_from_both_ends() -> Result<()> {
    use ValueType::{Tombstone, Value, ValueOverWeakTombstone, WeakTombstone};

    let mut random = generator(0x2545_f491_4f6c_dd1d);
    for _ in 0..200 {
        let mut items = Vec::new();
        let mut newest = Vec::new();
        for key in 0..6_u8 {
            for version in 0..random(4) {
                let value_type =
                    [Value, Tombstone, WeakTombstone, ValueOverWeakTombstone][random(4) as usize];
                let item =
                    InternalValue::from_components([key], [version as u8], 9 - version, value_type);
                if version == 0 {
                    newest.push(parts(item.clone()));
                }
                items.push(item);
            }
        }
        let stream = || MvccStream::new(items.clone().into_iter().map(Ok));

        let forward = stream()
            .map(|item| item.map(parts))
            .collect::<Result<Vec<_>>>()?;
        let reverse = stream()
            .rev()
            .map(|item| item.map(parts))
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(forward, newest);
        assert_eq!(reverse, newest.iter().cloned().rev().collect::<Vec<_>>());

        let (mut stream, mut front, mut back) = (stream(), Vec::new(), Vec::new());
        loop {
            let Some(item) = stream.next() else { break };
            front.push(parts(item?));
            let Some(item) = stream.next_back() else {
                break;
            };
            back.push(parts(item?));
        }
        front.extend(back.into_iter().rev());
        assert_eq!(front, newest);
    }

    let failing = || {
        MvccStream::new(
            [
                Ok(InternalValue::from_components("a", "new", 9, Value)),
                Err(Error::Io(std::io::Error::other("test error"))),
            ]
            .into_iter(),
        )
    };
    // The forward read drains the key's older versions, so it surfaces the error first.
    let mut stream = failing();
    assert!(matches!(stream.next(), Some(Err(Error::Io(_)))));
    assert!(stream.next().is_none() && stream.next_back().is_none());
    let mut stream = failing();
    assert!(matches!(stream.next_back(), Some(Err(Error::Io(_)))));
    assert!(matches!(stream.next_back(), Some(Ok(item)) if &*item.value == b"new"));
    assert!(stream.next().is_none() && stream.next_back().is_none());
    Ok(())
}

fn merge(items: Vec<InternalValue>, last_level: bool) -> Vec<InternalValue> {
    CompactionStream::new(items.into_iter().map(Ok))
        .evict_tombstones(last_level)
        .map(Result::unwrap)
        .collect()
}

fn version(seqno: u64, value_type: ValueType) -> InternalValue {
    let value: &[u8] = if value_type.is_tombstone() { b"" } else { b"v" };
    InternalValue::from_components("k", value, seqno, value_type)
}

fn types(items: &[InternalValue]) -> Vec<ValueType> {
    items.iter().map(|item| item.key.value_type).collect()
}

/// A reorg: an old value (V1, below the merge), its spend (W1), the rollback's restore (V2), the new spend
/// (W2). Whatever the merge grouping, the key must end deleted and V1 must not come back.
#[test]
fn restored_then_deleted_key_keeps_its_older_delete() {
    use ValueType::{Value as V, ValueOverWeakTombstone as P, WeakTombstone as W};
    // All three above V1 in one merge: the older weak tombstone carries on.
    assert_eq!(
        types(&merge(
            vec![version(4, W), version(3, V), version(2, W)],
            false
        )),
        [W]
    );
    // The restore meets the spend first: it keeps the pending weak tombstone...
    let merged = merge(vec![version(3, V), version(2, W)], false);
    assert_eq!(types(&merged), [P]);
    // ...which the new spend turns back into a weak tombstone, or resolves against V1.
    assert_eq!(
        types(&merge(vec![version(4, W), merged[0].clone()], false)),
        [W]
    );
    assert_eq!(
        types(&merge(vec![merged[0].clone(), version(1, V)], false)),
        [V]
    );
    // A key put twice, then deleted once, stays deleted; nothing pends at the last level.
    assert_eq!(
        types(&merge(
            vec![version(3, W), version(2, V), version(1, V)],
            false
        )),
        []
    );
    assert_eq!(types(&merge(vec![version(3, V), version(2, W)], true)), [V]);
}

/// Store histories as the indexer writes them (a key is put while absent, deleted while present, never deleted
/// and put again within one batch; e.g. outputs created, spent, restored by a rollback and spent again),
/// batched like `brk_store`'s pending map, with runs merged in arbitrary adjacent groups: every read must match a
/// plain map after every step.
#[test]
fn compaction_matches_a_map_for_any_merge_grouping() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut random = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };

    for _ in 0..300 {
        let mut map = BTreeMap::<u8, u64>::new();
        // Runs, newest first; each holds at most one version per key, sorted by key.
        let mut runs: Vec<Vec<InternalValue>> = Vec::new();
        let mut seqno = 0;

        for _ in 0..40 {
            if !runs.is_empty() && random(3) == 0 {
                let len = runs.len() as u64;
                let from = random(len) as usize;
                let to = from + random(len - from as u64) as usize;
                let last_level = to == runs.len() - 1;
                let mut items: Vec<InternalValue> = runs.drain(from..=to).flatten().collect();
                items.sort_by(|a, b| a.key.cmp(&b.key));
                let merged = merge(items, last_level);
                if !merged.is_empty() {
                    runs.insert(from, merged);
                }
            } else {
                let (mut puts, mut dels) = (BTreeMap::<u8, u64>::new(), BTreeSet::<u8>::new());
                for _ in 0..=random(4) {
                    let key = random(4) as u8;
                    if map.remove(&key).is_some() {
                        if puts.remove(&key).is_none() {
                            dels.insert(key);
                        }
                    } else if !dels.contains(&key) {
                        seqno += 1;
                        map.insert(key, seqno);
                        puts.insert(key, seqno);
                    }
                }
                seqno += 1;
                let mut run: Vec<InternalValue> = puts
                    .iter()
                    .map(|(key, value)| {
                        InternalValue::from_components(
                            [*key],
                            value.to_be_bytes(),
                            seqno,
                            ValueType::Value,
                        )
                    })
                    .chain(
                        dels.iter()
                            .map(|key| InternalValue::new_weak_tombstone([*key], seqno)),
                    )
                    .collect();
                run.sort_by(|a, b| a.key.cmp(&b.key));
                if !run.is_empty() {
                    runs.insert(0, run);
                }
            }

            for key in 0..4u8 {
                let read = runs
                    .iter()
                    .find_map(|run| {
                        run.iter()
                            .find(|item| item.key.user_key == [key].as_slice())
                    })
                    .filter(|item| !item.key.value_type.is_tombstone())
                    .map(|item| u64::from_be_bytes(item.value.as_ref().try_into().unwrap()));
                assert_eq!(read, map.get(&key).copied(), "key {key}, runs {runs:?}");
            }
        }
    }
}

// Runs

#[derive(Clone, Debug, Eq, PartialEq)]
struct FakeTable {
    id: u64,
    key_range: KeyRange,
}

impl Ranged for FakeTable {
    fn key_range(&self) -> &KeyRange {
        &self.key_range
    }
}

fn fake_table(id: u64, min: &[u8], max: &[u8]) -> FakeTable {
    FakeTable {
        id,
        key_range: KeyRange::new((min.into(), max.into())),
    }
}

#[test]
fn runs_find_tables_by_key_and_range() {
    let run = Run::new(vec![
        fake_table(0, b"a", b"d"),
        fake_table(1, b"e", b"j"),
        fake_table(2, b"k", b"o"),
        fake_table(3, b"p", b"z"),
    ])
    .unwrap();

    for (key, id) in [
        ("a", Some(0)),
        ("aaa", Some(0)),
        ("d", Some(0)),
        ("e", Some(1)),
        ("j", Some(1)),
        ("k", Some(2)),
        ("p", Some(3)),
        ("z", Some(3)),
        ("zzz", None),
    ] {
        assert_eq!(run.get_for_key(key.as_bytes()).map(|table| table.id), id);
    }

    use Bound::{Excluded, Included, Unbounded};
    let overlap = |lower: Bound<&str>, upper: Bound<&str>| {
        run.range_overlap_indexes::<&[u8], _>(&(lower.map(str::as_bytes), upper.map(str::as_bytes)))
    };
    assert_eq!(overlap(Unbounded, Unbounded), Some((0, 3)));
    assert_eq!(overlap(Included("a"), Included("a")), Some((0, 0)));
    assert_eq!(overlap(Included("a"), Excluded("d")), Some((0, 0)));
    assert_eq!(overlap(Included("d"), Included("d")), Some((0, 0)));
    assert_eq!(overlap(Included("a"), Included("g")), Some((0, 1)));
    assert_eq!(overlap(Excluded("d"), Excluded("e")), None);
    assert_eq!(overlap(Included("j"), Included("j")), Some((1, 1)));
    assert_eq!(overlap(Included("z"), Unbounded), Some((3, 3)));
    assert_eq!(overlap(Included("zzz"), Included("zzzz")), None);
}

/// A table must stay behind every newer run that overlaps it, or point reads find an older value
/// first. Generated layouts must match the one-table-at-a-time reference placement.
#[test]
fn optimize_runs_matches_reference_for_generated_layouts() {
    fn reference(runs: Vec<Run<FakeTable>>) -> Vec<Run<FakeTable>> {
        if runs.len() <= 1 {
            return runs;
        }
        let mut new_runs: Vec<Run<FakeTable>> = Vec::new();
        for run in runs {
            for table in run.into_inner() {
                let last_overlap = new_runs
                    .iter()
                    .rposition(|run| !run.get_overlapping(table.key_range()).is_empty());
                let target = match last_overlap {
                    Some(index) => new_runs.get_mut(index + 1),
                    None => new_runs.first_mut(),
                };
                if let Some(target) = target {
                    let tables = target.inner_mut();
                    let index = tables.partition_point(|existing| {
                        existing.key_range().min() < table.key_range().min()
                    });
                    tables.insert(index, table);
                } else {
                    new_runs.push(Run::new(vec![table]).unwrap());
                }
            }
        }
        new_runs
    }

    let mut random = generator(0x4d59_5df4_d0f3_3173);
    for case in 0..256_u64 {
        let run_count = 2 + (case as usize % 7);
        let runs: Vec<_> = (0..run_count)
            .map(|run_index| {
                let mut next_min = 0_u32;
                let tables = (0..8 + (case as usize + run_index) % 25)
                    .map(|table_index| {
                        let min = next_min + random(5) as u32;
                        let max = min + 1 + random(12) as u32;
                        next_min = max + 1;
                        fake_table(
                            (case << 32) | ((run_index as u64) << 16) | table_index as u64,
                            &min.to_be_bytes(),
                            &max.to_be_bytes(),
                        )
                    })
                    .collect();
                Run::new(tables).unwrap()
            })
            .collect();
        assert_eq!(
            reference(runs.clone()),
            optimize_runs(runs),
            "generated case {case}"
        );
    }
}

// Persistence and recovery

#[test]
fn manifests_recover_entries_and_reject_corruption() -> Result<()> {
    fn write_manifest(directory: &Path, table_count: u32, payload: impl Fn(&mut Vec<u8>)) {
        let mut current = MAGIC_BYTES.to_vec();
        current.write_u64::<LittleEndian>(7).unwrap();
        current.write_u8(1).unwrap();
        current.write_u32::<LittleEndian>(table_count).unwrap();
        payload(&mut current);
        for _ in 1..DEFAULT_LEVEL_COUNT {
            current.write_u8(0).unwrap();
        }
        let checksum = xxh3::xxh3_128(&current);
        current.write_u128::<LittleEndian>(checksum).unwrap();
        fs::write(directory.join(CURRENT_VERSION_FILE), current).unwrap();
    }

    let directory = tempdir()?;
    write_manifest(directory.path(), 1, |current| {
        current.write_u32::<LittleEndian>(42).unwrap();
        current.write_u64::<LittleEndian>(84).unwrap();
    });
    let recovery = Recovery::load(directory.path())?;
    let table = &recovery.table_ids[0][0][0];
    assert_eq!(
        (recovery.curr_version_id, table.id, table.global_seqno),
        (7, 42, 84)
    );

    // A huge table count is rejected before allocating.
    write_manifest(directory.path(), u32::MAX, |_| {});
    assert!(matches!(
        Recovery::load(directory.path()),
        Err(Error::Unrecoverable)
    ));

    // Persisting replaces a partial manifest; a flipped payload byte fails the checksum.
    let path = directory.path().join(CURRENT_VERSION_FILE);
    fs::write(&path, b"partial")?;
    Version::new(7).persist(directory.path())?;
    assert_eq!(Recovery::load(directory.path())?.curr_version_id, 7);
    let mut current = fs::read(&path)?;
    current[MAGIC_BYTES.len() + size_of::<u64>()] ^= 1;
    fs::write(&path, current)?;
    assert!(matches!(
        Recovery::load(directory.path()),
        Err(Error::Unrecoverable)
    ));
    Ok(())
}

#[test]
fn trees_refuse_truncated_or_old_manifests_without_rewriting() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    let mut ingestion = tree.ingestion()?;
    ingestion.write("a", "1")?;
    ingestion.finish()?;
    drop(tree);

    let path = directory.path().join(CURRENT_VERSION_FILE);
    let mut bytes = fs::read(&path)?;
    for version in [8, 9, 10] {
        bytes[3] = version;
        fs::write(&path, &bytes)?;
        assert!(matches!(
            open(directory.path()),
            Err(Error::InvalidVersion(found)) if found == version
        ));
        assert_eq!(fs::read(&path)?, bytes);
    }

    bytes[3] = MAGIC_BYTES[3];
    bytes.truncate(5);
    fs::write(&path, bytes)?;
    assert!(matches!(open(directory.path()), Err(Error::Unrecoverable)));
    Ok(())
}

#[test]
fn files_read_exactly_and_rewrite_atomically() -> Result<()> {
    let mut file = NamedTempFile::new()?;
    let bytes: Vec<_> = (0..=255_u8).cycle().take(8192).collect();
    file.write_all(&bytes)?;
    for len in [0, 8, 12, 13, 4096] {
        assert_eq!(
            read_exact(file.as_file(), 7, len)?.as_ref(),
            &bytes[7..7 + len]
        );
    }
    assert!(matches!(
        read_exact(file.as_file(), 8190, 16),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof
    ));

    let directory = tempdir()?;
    let path = directory.path().join("file");
    fs::write(&path, b"old content")?;
    rewrite_atomic(&path, b"new")?;
    assert_eq!(fs::read(&path)?, b"new");
    let mut temp_file = NamedTempFile::new_in(directory.path())?;
    temp_file.write_all(b"newer")?;
    persist_temp_file(temp_file, &path)?;
    assert_eq!(fs::read(&path)?, b"newer");
    Ok(())
}

// Trees

#[test]
fn ingestion_reads_and_recovery() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;

    // An empty ingestion publishes nothing and leaves no table file behind.
    let version = tree.current_version_id();
    tree.ingestion()?.finish()?;
    assert_eq!(tree.current_version_id(), version);
    assert_eq!(fs::read_dir(directory.path().join("tables"))?.count(), 0);

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

    let check = |tree: &Tree| -> Result<()> {
        let expected = [
            (b"a".to_vec(), b"1".to_vec()),
            (b"b".to_vec(), b"20".to_vec()),
            (b"d".to_vec(), b"4".to_vec()),
        ];
        assert_eq!(tree.get("a")?.as_deref(), Some(b"1".as_slice()));
        assert_eq!(tree.get("b")?.as_deref(), Some(b"20".as_slice()));
        assert_eq!(tree.get("c")?, None);
        assert_eq!(collect(tree.iter())?, expected);
        assert_eq!(collect(tree.range("b"..="d"))?, expected[1..]);
        assert_eq!(collect(tree.prefix("b"))?, expected[1..2]);
        Ok(())
    };
    check(&tree)?;
    drop(tree);

    // The recovered table-id counter must not reuse a published table's id.
    let tree = open(directory.path())?;
    check(&tree)?;
    let mut ingestion = tree.ingestion()?;
    ingestion.write("e", "5")?;
    ingestion.finish()?;
    drop(tree);

    let tree = open(directory.path())?;
    assert_eq!(tree.get("e")?.as_deref(), Some(b"5".as_slice()));
    assert_eq!(tree.get("a")?.as_deref(), Some(b"1".as_slice()));
    assert_eq!(tree.get("b")?.as_deref(), Some(b"20".as_slice()));
    Ok(())
}

#[test]
fn transitive_overlap_preserves_newest_value_after_recovery() -> Result<()> {
    let directory = tempdir()?;
    let tree = open(directory.path())?;
    for batch in [
        &[("a", "oldest"), ("c", "oldest")][..],
        &[("a", "middle"), ("n", "middle"), ("z", "middle")],
        &[("m", "newest"), ("n", "newest"), ("p", "newest")],
    ] {
        let mut ingestion = tree.ingestion()?;
        for (key, value) in batch {
            ingestion.write(*key, *value)?;
        }
        ingestion.finish()?;
    }

    assert_eq!(tree.get("n")?.as_deref(), Some(b"newest".as_slice()));
    drop(tree);
    assert_eq!(
        open(directory.path())?.get("n")?.as_deref(),
        Some(b"newest".as_slice())
    );
    Ok(())
}

#[test]
#[should_panic(expected = "ingestion keys must be strictly increasing")]
fn ingestion_rejects_unsorted_keys_in_debug_builds() {
    let directory = tempdir().unwrap();
    let tree = open(directory.path()).unwrap();
    let mut ingestion = tree.ingestion().unwrap();
    ingestion.write("b", "1").unwrap();
    ingestion.write("a", "2").unwrap();
}

#[test]
fn ingestion_honors_policies_before_and_after_recovery() -> Result<()> {
    for expect_point_read_hits in [false, true] {
        for (pin_filter, pin_index) in [(false, false), (true, false), (false, true), (true, true)]
        {
            let directory = tempdir()?;
            let config = || {
                Config::new(directory.path())
                    .expect_point_read_hits(expect_point_read_hits)
                    .filter_block_pinning_policy(PinningPolicy::all(pin_filter))
                    .index_block_pinning_policy(PinningPolicy::all(pin_index))
            };
            let mut tree = config().open()?;
            let mut ingestion = tree.ingestion_as::<[u8; 8], [u8; 4]>()?;
            ingestion.write([1; 8], [2; 4])?;
            ingestion.finish()?;

            for recovered in [false, true] {
                if recovered {
                    drop(tree);
                    tree = config().open()?;
                }
                let version = tree.versions.guard();
                for table in version
                    .iter_levels()
                    .flat_map(Level::iter)
                    .flat_map(|run| run.iter())
                {
                    assert!(table.filter_size() > 0);
                    assert_eq!(table.pinned_filter_size() > 0, pin_filter);
                    assert_eq!(table.pinned_block_index_size() > 0, pin_index);
                }
                assert_eq!(tree.get_as::<[u8; 4]>(&[1; 8])?, Some([2; 4]));
            }
        }
    }
    Ok(())
}

/// Bounded scans over many data blocks match a sorted reference for every index layout, in both
/// directions and interleaved, with bounds on, between and beyond keys and tombstones.
#[test]
fn ranges_match_a_sorted_reference_across_index_layouts() -> Result<()> {
    const COUNT: u32 = 40_000;
    let key = |i: u32| {
        let mut key = (i * 2).to_be_bytes().to_vec();
        key.extend(std::iter::repeat_n(0xab, (i % 5) as usize));
        key
    };
    let gap = |i: u32| (i * 2 + 1).to_be_bytes().to_vec();
    let value = |i: u32| vec![i as u8; (i % 37) as usize];
    let is_tombstone = |i: u32| i % 11 == 5;
    let expected: Vec<_> = (0..COUNT)
        .filter(|i| !is_tombstone(*i))
        .map(|i| (key(i), value(i)))
        .collect();

    let mut random = generator(0x9e37_79b9_7f4a_7c15);
    for (pinned, partitioned, restart_interval) in [
        (false, false, 1),
        (true, false, 4),
        (false, true, 16),
        (true, true, 8),
    ] {
        let directory = tempdir()?;
        let config = || {
            Config::new(directory.path())
                .data_block_restart_interval_policy(RestartIntervalPolicy::all(restart_interval))
                .index_block_pinning_policy(PinningPolicy::all(pinned))
                .filter_block_pinning_policy(PinningPolicy::all(pinned))
                .index_block_partitioning_policy(PartitioningPolicy::all(partitioned))
                .filter_block_partitioning_policy(PartitioningPolicy::all(partitioned))
        };
        let tree = config().open()?;
        let mut ingestion = tree.ingestion()?;
        for i in 0..COUNT {
            if is_tombstone(i) {
                ingestion.write_weak_tombstone(key(i))?;
            } else {
                ingestion.write(key(i), value(i))?;
            }
        }
        ingestion.finish()?;

        for reopen in [false, true] {
            let tree = if reopen {
                config().open()?
            } else {
                tree.clone()
            };
            for i in (0..COUNT).step_by(7) {
                assert_eq!(
                    tree.get(key(i))?.map(|value| value.to_vec()),
                    (!is_tombstone(i)).then(|| value(i))
                );
                assert!(tree.get(gap(i))?.is_none());
            }

            let check = |lower: Bound<Vec<u8>>, upper: Bound<Vec<u8>>| -> Result<()> {
                let wanted: Vec<_> = expected
                    .iter()
                    .filter(|(key, _)| (lower.as_ref(), upper.as_ref()).contains(key))
                    .cloned()
                    .collect();
                let range = || tree.range::<Vec<u8>, _>((lower.clone(), upper.clone()));

                assert_eq!(collect(range())?, wanted);
                assert_eq!(
                    collect(range().rev())?,
                    wanted.iter().rev().cloned().collect::<Vec<_>>()
                );
                let (mut iter, mut front, mut back) = (range(), Vec::new(), Vec::new());
                loop {
                    let Some(item) = iter.next() else { break };
                    front.push(item?);
                    let Some(item) = iter.next_back() else { break };
                    back.push(item?);
                }
                let interleaved: Vec<_> = front
                    .into_iter()
                    .chain(back.into_iter().rev())
                    .map(|(key, value)| (key.to_vec(), value.to_vec()))
                    .collect();
                assert_eq!(interleaved, wanted);
                Ok(())
            };

            // Every key of a window spanning several data blocks as both bounds, so each
            // block boundary is hit inclusively and exclusively.
            for i in 1_000..1_400 {
                check(Bound::Included(key(i)), Bound::Excluded(key(i + 3)))?;
                check(Bound::Excluded(key(i)), Bound::Included(key(i + 3)))?;
            }

            for _ in 0..150 {
                let start = random(u64::from(COUNT) + 1) as u32;
                let end = start + random(800) as u32;
                let mut bound = |index: u32| {
                    let bound_key = if random(2) == 0 && index < COUNT {
                        key(index)
                    } else {
                        gap(index.saturating_sub(1))
                    };
                    match random(7) {
                        0 => Bound::Unbounded,
                        1..=3 => Bound::Included(bound_key),
                        _ => Bound::Excluded(bound_key),
                    }
                };
                let mut lower = bound(start);
                let mut upper = bound(end);
                if let (
                    Bound::Included(low) | Bound::Excluded(low),
                    Bound::Included(high) | Bound::Excluded(high),
                ) = (&mut lower, &mut upper)
                    && low > high
                {
                    std::mem::swap(low, high);
                }
                check(lower, upper)?;
            }
        }
    }
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
    assert!(tree.l0_run_count() < 8);
    assert_eq!(collect(reader)?.len(), 63);

    let check = |tree: &Tree| -> Result<()> {
        assert_eq!(collect(tree.iter())?.len(), 63);
        for key in 0..64_u8 {
            let expected = (key != 10).then_some([7_u8]);
            assert_eq!(
                tree.get([key])?.as_deref(),
                expected.as_ref().map(<[u8; 1]>::as_slice)
            );
        }
        Ok(())
    };
    check(&tree)?;
    drop(tree);
    check(&open(directory.path())?)
}

#[test]
fn fixed_records_preserve_values_tombstones_and_order() -> Result<()> {
    fn check<const K: usize, const V: usize>(fixed_compaction: bool) -> Result<()> {
        let dir = tempdir()?;
        let config = Config::new(dir.path())
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
    for fixed_compaction in [false, true] {
        check::<8, 4>(fixed_compaction)?;
        check::<8, 0>(fixed_compaction)?;
        check::<10, 0>(fixed_compaction)?;
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
