#[allow(dead_code)]
mod common;

use std::sync::Arc;

use bitview_vecs::LazyIndexedVec;
use brk_types::{Height, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, Ident, ImportableVec, LazyVec, MutableVec, ReadOnlyClone,
    ReadableCloneableVec, ReadableVec, WritableVec,
};

#[test]
fn indexed_chunks_preserve_captures_offsets_short_metadata_and_rewrites() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Height, _>(
        &db,
        "source",
        (0..40_000u64).map(|i| StoredU64::from(i * 3)),
    );
    let mut metadata = common::stored::<Height, _>(
        &db,
        "metadata",
        (0..35_000u64).map(|i| StoredU64::from(i * 5)),
    );
    let cached_metadata = metadata.read_only_clone();
    let factor = Arc::new(7u64);
    let indexed = LazyIndexedVec::new(
        "indexed",
        Version::ONE,
        &source,
        &cached_metadata,
        move |height: Height, source: StoredU64, metadata: StoredU64| {
            StoredU64::from(
                u64::from(source) + u64::from(metadata) * *factor + usize::from(height) as u64,
            )
        },
    );
    let identity = LazyVec::<Height, StoredU64, Height, StoredU64>::transformed::<Ident>(
        "identity",
        Version::ONE,
        indexed.read_only_boxed_clone(),
    );
    for (from, to) in [
        (0, 40_000),
        (16_000, 33_000),
        (34_990, 40_000),
        (7, 2),
        (usize::MAX, usize::MAX),
    ] {
        let expected: Vec<_> = (from..to.min(35_000))
            .map(|i| StoredU64::from(i as u64 * 39))
            .collect();
        let mut chunks = Vec::new();
        indexed.for_each_chunk_at(from, to, &mut |at, values| {
            assert_eq!(at, from + chunks.len());
            chunks.extend_from_slice(values);
        });
        assert_eq!(chunks, expected);
        assert_eq!(indexed.collect_range_at(from, to), expected);
        assert_eq!(identity.clone().collect_range_at(from, to), expected);
        let mut visited = Vec::new();
        indexed.for_each_range_dyn_at(from, to, &mut |value| visited.push(value));
        assert_eq!(visited, expected);
        let mut appended = vec![StoredU64::from(123u64)];
        identity.read_into_at(from, to, &mut appended);
        assert_eq!(&appended[1..], expected);
    }
    let mut calls = 0;
    let result = identity.try_fold_range_at(16_000, 33_000, (), |(), value| {
        assert_eq!(u64::from(value), (16_000 + calls) * 39);
        calls += 1;
        if calls == 3 { Err("stop") } else { Ok(()) }
    });
    assert_eq!(result, Err("stop"));
    assert_eq!(calls, 3);

    metadata.truncate_if_needed_at(34_999).unwrap();
    metadata.push(StoredU64::from(1u64));
    metadata.write().unwrap();
    assert_eq!(
        identity.collect_range_at(34_999, 40_000),
        [StoredU64::from(34_999u64 * 4 + 7)]
    );
}

#[test]
fn indexed_chunks_keep_metadata_aligned_with_emitted_values_across_holes() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut source =
        MutableVec::<BytesVec<usize, u64>>::import(&db, "source", Version::ONE).unwrap();
    let mut metadata = BytesVec::<usize, u64>::import(&db, "metadata", Version::ONE).unwrap();
    for i in 0..20_000u64 {
        source.push(i * 3);
    }
    for i in 0..18_000u64 {
        metadata.push(i * 5);
    }
    for i in [1, 4095, 8192, 16_383] {
        source.delete_at(i);
    }
    for i in 4096..8192 {
        source.delete_at(i);
    }
    source.write().unwrap();
    metadata.write().unwrap();
    let indexed = LazyIndexedVec::new(
        "indexed",
        Version::ONE,
        &source,
        &metadata.read_only_clone(),
        |index: usize, value, weight| value + weight * 7 + index as u64,
    );
    for (from, to) in [(0, 20_000), (3990, 9000), (4096, 8192), (17_000, 20_000)] {
        let expected: Vec<_> = source
            .collect_range_at(from, to.min(18_000))
            .into_iter()
            .enumerate()
            .map(|(offset, value)| value + (from + offset) as u64 * 36)
            .collect();
        assert_eq!(indexed.collect_range_at(from, to), expected);
        let mut actual = Vec::new();
        indexed.for_each_chunk_at(from, to, &mut |at, values| {
            assert_eq!(at, from + actual.len());
            actual.extend_from_slice(values);
        });
        assert_eq!(actual, expected);
        assert_eq!(
            indexed.fold_range_at(from, to, Vec::new(), |mut values, value| {
                values.push(value);
                values
            }),
            expected
        );
    }
}
