use brk_types::{Height, StoredU64, Version};
use vecdb::{
    AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec, ReadableVec,
    WritableVec,
};

use super::{LazyCumulativeIndexVec, LazyIndexCountVec};

#[test]
fn next_boundaries_produce_cumulative_and_per_item_counts() {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brk-next-index-{}-{suffix}", std::process::id()));
    let db = Database::open(&path).unwrap();
    let mut first: EagerVec<PcoVec<Height, Height>> =
        EagerVec::forced_import(&db, "first", Version::ONE).unwrap();
    let mut terminal: EagerVec<PcoVec<Height, StoredU64>> =
        EagerVec::forced_import(&db, "terminal", Version::ONE).unwrap();

    for value in [0, 2, 5] {
        first.push(Height::new(value));
    }
    for value in 0_u64..6 {
        terminal.push(StoredU64::from(value));
    }
    first.write().unwrap();
    terminal.write().unwrap();

    let cumulative = LazyCumulativeIndexVec::new(
        "cumulative",
        Version::ONE,
        first.read_only_boxed_clone(),
        terminal.read_only_boxed_clone(),
    );
    let count = LazyIndexCountVec::new(
        "count",
        Version::ONE,
        first.read_only_boxed_clone(),
        terminal.read_only_boxed_clone(),
    );

    assert_eq!(
        cumulative.collect_range(Height::ZERO, Height::new(3)),
        [2_u64, 5, 6].map(StoredU64::from)
    );
    assert_eq!(
        count.collect_range(Height::ZERO, Height::new(3)),
        [2_u64, 3, 1].map(StoredU64::from)
    );
    assert_eq!(
        count.collect_range(Height::new(1), Height::new(3)),
        [3_u64, 1].map(StoredU64::from)
    );
    assert_eq!(
        cumulative.collect_one(Height::new(2)),
        Some(StoredU64::new(6))
    );
    assert_eq!(count.collect_one(Height::new(2)), Some(StoredU64::new(1)));
    assert_eq!(
        cumulative.read_sorted_at(&[0, 2, 2, 3]),
        [2_u64, 6, 6].map(StoredU64::from)
    );
    assert_eq!(
        count.read_sorted_at(&[0, 2, 2, 3]),
        [2_u64, 1, 1].map(StoredU64::from)
    );

    drop(count);
    drop(cumulative);
    drop(first);
    drop(terminal);
    drop(db);
    std::fs::remove_dir_all(path).unwrap();
}
