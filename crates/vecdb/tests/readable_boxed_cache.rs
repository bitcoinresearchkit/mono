use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, CachedVec, Database, EagerVec, ImportableVec, LazyVec,
    ReadableBoxedVec, ReadableVec, StoredVec, Version, WritableVec,
};

#[test]
fn boxed_vec_detects_only_actual_cache_layers() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let mut values: EagerVec<BytesVec<usize, u64>> =
        EagerVec::import(&db, "values", Version::ONE).unwrap();

    for value in 0..8 {
        values.push(value);
    }
    values.write().unwrap();

    let read_only = values.read_only_clone();
    let uncached = ReadableBoxedVec::new(read_only.clone());
    assert!(!uncached.has_cache_layer());

    let cached = ReadableBoxedVec::new(CachedVec::wrap(read_only));
    assert!(cached.has_cache_layer());
    assert!(cached.clone().has_cache_layer());
    assert_eq!(cached.read_sorted_at(&[0, 2, 7]), [0, 2, 7]);

    let lazy =
        LazyVec::<usize, u64, usize, u64>::init("lazy_values", Version::ONE, cached, |_, value| {
            value
        });
    assert!(!lazy.has_cache_layer());
}
