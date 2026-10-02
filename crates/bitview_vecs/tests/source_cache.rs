use common::init_cache;

use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use brk_types::{Height, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, PcoVec, ReadOnlyClone, ReadableVec, WritableVec,
};

#[allow(dead_code)]
mod common;

#[test]
fn height_owner_catalog_and_read_only_clone_share_one_budgeted_cache() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let mut metric =
        PerBlock::<StoredU64>::forced_import(&db, "source_cache_height", Version::ONE, &indexes)
            .unwrap();
    let _: &EagerVec<PcoVec<Height, StoredU64, Budgeted>> = &metric.height;
    for i in 0..4096_u64 {
        metric.height.push(StoredU64::from(i));
    }
    metric.height.write().unwrap();
    let reader = metric.read_only_clone();
    assert!(!metric.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    assert_eq!(
        reader.height.collect_last(),
        Some(StoredU64::from(4095_u64))
    );
    assert!(
        metric
            .height
            .read_cached_into_at(4095, 4096, &mut Vec::new())
    );
    assert!(!metric.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    let mut json = Vec::new();
    reader
        .iter_any_exportable()
        .find(|v| v.name() == "source_cache_height")
        .unwrap()
        .write_json(None, None, &mut json)
        .unwrap();
    assert!(json.starts_with(b"[0,1,2,"));
    let mut original = Vec::new();
    assert!(metric.height.read_cached_into_at(0, 4096, &mut original));
    metric.height.truncate_if_needed_at(4096).unwrap();
    metric.height.push(StoredU64::from(4096_u64));
    metric.height.write().unwrap();
    assert!(reader.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    assert!(
        reader
            .height
            .read_cached_into_at(4096, 4097, &mut Vec::new())
    );
    assert_eq!(
        reader.height.collect_last(),
        Some(StoredU64::from(4096_u64))
    );
    metric.height.truncate_if_needed_at(4095).unwrap();
    metric.height.push(StoredU64::from(9000_u64));
    metric.height.push(StoredU64::from(9001_u64));
    metric.height.write().unwrap();
    assert!(reader.height.read_cached_into_at(0, 4095, &mut Vec::new()));
    assert!(
        reader
            .height
            .read_cached_into_at(4095, 4097, &mut Vec::new())
    );
    assert_eq!(
        reader.height.collect_range_at(4095, 4097),
        [StoredU64::from(9000_u64), StoredU64::from(9001_u64)]
    );
    assert_eq!(original[4095], StoredU64::from(4095_u64));
}
