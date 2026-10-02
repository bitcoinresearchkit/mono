use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, BytesVec, Database, ImportableVec, ReadableCloneableVec, ReadableVec,
    Version, WritableVec,
};

#[test]
fn stored_policies_share_retained_ranges_through_clones() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let budget = Budgeted::init_global(4096).unwrap();
    let mut source =
        BytesVec::<usize, u64, Budgeted>::import(&db, "budgeted", Version::ONE).unwrap();
    for value in 10..30 {
        source.push(value);
    }
    source.write().unwrap();
    let read_only = source.read_only_clone();
    let boxed = read_only.read_only_boxed_clone();
    assert!(!boxed.read_cached_into_at(2, 4, &mut Vec::new()));
    assert_eq!(source.collect_range_at(2, 4), [12, 13]);
    for source in [&read_only as &dyn ReadableVec<usize, u64>, &boxed] {
        let mut out = vec![99];
        assert!(source.read_cached_into_at(2, 4, &mut out));
        assert_eq!(out, [99, 12, 13]);
        assert!(!source.read_cached_into_at(1, 5, &mut out));
        assert_eq!(
            out,
            [99, 12, 13],
            "a miss leaves the caller buffer unchanged"
        );
    }
    assert!(budget.used() > 0 && budget.used() <= budget.limit());
    budget.clear();
    assert_eq!(budget.used(), 0);
    assert!(!boxed.read_cached_into_at(2, 4, &mut Vec::new()));
    assert_eq!(boxed.collect_range_dyn(2, 4), [12, 13]);

    // A computation owns ordinary read results. Their lifetime is independent
    // of cache eviction, and its next source read can repopulate the cache.
    let working_values = source.collect_range_at(2, 4);
    budget.clear();
    assert_eq!(budget.used(), 0);
    assert_eq!(working_values, [12, 13]);
    assert_eq!(source.collect_range_at(2, 4), working_values);
}
