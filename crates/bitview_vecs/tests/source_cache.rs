use common::init_cache;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use bitview_transforms::RatioU64;
use bitview_traversable::Traversable;
use bitview_vecs::{DailyMappings, DailyMetric, LazyPercentPerBlock, PerBlock, Resolutions};
use brk_exit::Exit;
use brk_types::{Day1, Height, PartsPerMillion32, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, LazyVec, PcoVec, ReadOnlyClone,
    ReadableCloneableVec, ReadableVec, WritableVec,
};

#[allow(dead_code)]
mod common;

#[test]
fn ratio_reads_reuse_stored_counts_without_retaining_derived_history() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let counts =
        common::stored::<Height, _>(&db, "counts", (1..=16).map(|i| StoredU64::from(i * 3u64)));
    static READS: AtomicUsize = AtomicUsize::new(0);
    let counted = LazyVec::init(
        "counted",
        Version::ONE,
        counts.read_only_boxed_clone(),
        |_, value| {
            READS.fetch_add(1, Relaxed);
            value
        },
    );
    let denominator = common::stored::<Height, _>(
        &db,
        "denominator",
        (1..=16).map(|i| StoredU64::from(i * 7u64)),
    );
    let ratio = LazyPercentPerBlock::from_ratio::<_, _, RatioU64<PartsPerMillion32>>(
        "share",
        Version::ONE,
        &counted,
        &denominator,
        &indexes,
    );
    for _ in 0..2 {
        READS.store(0, Relaxed);
        assert_eq!(
            ratio.ppm.height.collect(),
            vec![PartsPerMillion32::from(3.0 / 7.0); 16]
        );
        assert!(
            READS.load(Relaxed) > 0,
            "derived source acquired its own cache"
        );
    }
}

#[test]
fn generic_clones_share_source_ranges_without_retaining_derived_histories() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut source =
        EagerVec::<PcoVec<Height, StoredU64, Budgeted>>::import(&db, "generic", Version::ONE)
            .unwrap();
    for i in 0..4096_u64 {
        source.push(StoredU64::from(i));
    }
    source.write().unwrap();
    let expected = source.collect();
    let reader = source.read_only_clone();
    let erased = source.read_only_boxed_clone();
    let indexes = common::indexes(&db);
    let resolutions =
        Resolutions::from_source("generic_resolutions", &source, Version::ONE, &indexes);
    for source in [
        &reader as &dyn ReadableVec<Height, StoredU64>,
        &erased,
        resolutions.height_source(),
    ] {
        let mut values = Vec::new();
        assert!(source.read_cached_into_at(17, 100, &mut values));
        assert_eq!(values, expected[17..100]);
    }
    let derived = LazyVec::init("double", Version::ONE, erased, |_: Height, value| {
        StoredU64::from(u64::from(value) * 2)
    });
    assert_eq!(derived.collect_last(), Some(StoredU64::from(8190_u64)));
    assert!(!derived.read_cached_into_at(0, 4096, &mut Vec::new()));

    let mut plain = PcoVec::<Height, StoredU64>::import(&db, "uncached", Version::ONE).unwrap();
    for &value in &expected {
        plain.push(value);
    }
    plain.write().unwrap();
    assert_eq!(plain.collect(), expected);
    assert!(
        !plain
            .read_only_boxed_clone()
            .read_cached_into_at(0, 4096, &mut Vec::new())
    );
}

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

#[test]
fn compute_helpers_share_the_same_source_owner() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let mut metric =
        PerBlock::<StoredU64>::forced_import(&db, "source_cache_compute", Version::ONE, &indexes)
            .unwrap();
    metric.height.push(StoredU64::from(1_u64));
    metric.height.write().unwrap();
    let reader = metric.read_only_clone();
    assert_eq!(reader.height.collect(), [StoredU64::from(1_u64)]);
    let source: &mut EagerVec<PcoVec<Height, StoredU64, Budgeted>> = &mut metric.height;
    source.truncate_if_needed_at(0).unwrap();
    source.push(StoredU64::from(2_u64));
    source.write().unwrap();
    assert!(!reader.height.read_cached_into_at(0, 1, &mut Vec::new()));
    assert_eq!(reader.height.collect_last(), Some(StoredU64::from(2_u64)));
}

#[test]
fn daily_views_retain_only_their_requested_points_and_catalog_reuses_them() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.height_day1 = common::stored(
        &db,
        "daily_view_mapping",
        [Day1::from(0usize), Day1::from(4095usize)],
    )
    .read_only_boxed_clone();
    let mappings = DailyMappings::new(&indexes);
    let mut metric =
        DailyMetric::<StoredU64>::forced_import(&db, "source_cache_day", Version::ONE, &mappings)
            .unwrap();
    let _: &EagerVec<PcoVec<Day1, StoredU64, Budgeted>> = &metric.day1;
    for i in 0..4096_u64 {
        metric.day1.push(StoredU64::from(i));
    }
    metric.day1.write().unwrap();
    assert_eq!(
        [
            metric.views.height.collect_one_at(0).unwrap(),
            metric.views.height.collect_one_at(1).unwrap(),
        ],
        [
            Some(StoredU64::from(0_u64)),
            Some(StoredU64::from(4095_u64))
        ]
    );
    assert!(metric.day1.read_cached_into_at(0, 1, &mut Vec::new()));
    assert!(metric.day1.read_cached_into_at(4095, 4096, &mut Vec::new()));
    assert!(!metric.day1.read_cached_into_at(0, 4096, &mut Vec::new()));
    let reader = metric.read_only_clone();
    let mut json = Vec::new();
    reader
        .iter_any_exportable()
        .find(|v| v.name() == "source_cache_day")
        .unwrap()
        .write_json(None, None, &mut json)
        .unwrap();
    assert!(metric.day1.read_cached_into_at(0, 4096, &mut Vec::new()));
    metric
        .day1
        .truncate_if_needed(Day1::from(4095usize))
        .unwrap();
    metric.day1.push(StoredU64::from(7000_u64));
    metric.day1.write().unwrap();
    assert_eq!(reader.day1.collect_last(), Some(StoredU64::from(7000_u64)));
    assert_eq!(
        reader.views.height.collect_last(),
        Some(Some(StoredU64::from(7000_u64)))
    );
}

#[test]
fn incremental_compute_preserves_cached_prefixes_and_invalidates_rewrites() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let mut source =
        PerBlock::<StoredU64>::forced_import(&db, "incremental_source", Version::ONE, &indexes)
            .unwrap();
    let mut target =
        PerBlock::<StoredU64>::forced_import(&db, "incremental_target", Version::ONE, &indexes)
            .unwrap();
    for i in 0..4096_u64 {
        source.height.push(StoredU64::from(i));
    }
    source.height.write().unwrap();
    let exit = Exit::new();
    let compute = |from: usize, target: &mut PerBlock<StoredU64>, source: &PerBlock<StoredU64>| {
        let mut count = 0;
        target
            .height
            .compute_transform(
                Height::from(from),
                &source.height,
                |(height, value, _)| {
                    count += 1;
                    (height, value)
                },
                &exit,
            )
            .unwrap();
        count
    };
    assert_eq!(compute(0, &mut target, &source), 4096);
    let reader = target.read_only_clone();
    let original = reader.height.collect();
    assert!(source.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    source.height.push(StoredU64::from(4096_u64));
    source.height.write().unwrap();
    assert_eq!(compute(4096, &mut target, &source), 1);
    assert!(source.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    assert!(reader.height.read_cached_into_at(0, 4096, &mut Vec::new()));
    assert_eq!(
        reader.height.collect_last(),
        Some(StoredU64::from(4096_u64))
    );
    source.height.truncate_if_needed_at(4095).unwrap();
    source.height.push(StoredU64::from(8000_u64));
    source.height.push(StoredU64::from(8001_u64));
    source.height.write().unwrap();
    assert_eq!(compute(4095, &mut target, &source), 2);
    assert!(reader.height.read_cached_into_at(0, 4095, &mut Vec::new()));
    assert!(
        reader
            .height
            .read_cached_into_at(4095, 4097, &mut Vec::new())
    );
    assert_eq!(
        reader.height.collect_last(),
        Some(StoredU64::from(8001_u64))
    );
    assert_eq!(original[4095], StoredU64::from(4095_u64));
}
