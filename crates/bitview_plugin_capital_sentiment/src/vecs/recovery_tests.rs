use super::compute_series;
use bitview_vecs::import_cached;
use brk_exit::Exit;
use brk_types::{Cents, Height, StoredBool, StoredU8};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, Database, ImportableVec, PcoVec, ReadOnlyClone, ReadableVec,
    Version, WritableVec,
};
#[path = "../../../bitview_vecs/tests/common/cache.rs"]
mod cache;

#[test]
fn signals_recover_partial_groups_reorgs_revisions_and_zero_append_rewinds() {
    cache::init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut source = [
        ("spot", [40u64, 60, 100, 40]),
        ("all", [70; 4]),
        ("sth", [80; 4]),
        ("lth", [60; 4]),
        ("sma", [90; 4]),
    ]
    .map(|(name, values)| {
        let mut vec = import_cached::<Height, Cents>(&db, name, Version::ONE).unwrap();
        for value in values {
            vec.push(Cents::new(value));
        }
        vec.write().unwrap();
        vec
    });
    let mut phase = import_cached::<Height, StoredU8>(&db, "phase", Version::ONE).unwrap();
    let mut position = import_cached::<Height, StoredBool>(&db, "position", Version::ONE).unwrap();
    let exit = Exit::new();

    compute_series(&mut phase, &mut position, sources(&source), 4, &exit).unwrap();
    assert_eq!(
        position.collect(),
        [false, false, true, false].map(StoredBool::from)
    );
    let expected = (phase.collect(), position.collect());
    phase.truncate_if_needed_at(3).unwrap();
    phase.write().unwrap();
    compute_series(&mut phase, &mut position, sources(&source), 4, &exit).unwrap();
    assert_eq!((phase.collect(), position.collect()), expected);
    source[0].truncate_if_needed_at(3).unwrap();
    source[0].push(Cents::new(100));
    source[0].write().unwrap();
    compute_series(&mut phase, &mut position, sources(&source), 3, &exit).unwrap();
    assert_eq!(
        position.collect(),
        [false, false, true, true].map(StoredBool::from)
    );
    source[0]
        .validate_computed_version_or_reset(Version::new(101))
        .unwrap();
    for _ in 0..4 {
        source[0].push(Cents::new(100));
    }
    source[0].write().unwrap();
    compute_series(&mut phase, &mut position, sources(&source), 4, &exit).unwrap();
    assert_eq!(position.collect(), [StoredBool::FALSE; 4]);
    source[0].truncate_if_needed_at(3).unwrap();
    source[0].write().unwrap();
    let reader = phase.read_only_clone();
    compute_series(&mut phase, &mut position, sources(&source), 4, &exit).unwrap();
    assert_eq!(reader.len(), 3);
    assert_eq!(position.len(), 3);
    let expected = (phase.collect(), position.collect());
    drop(reader);
    drop(phase);
    drop(position);
    drop(source);
    db.flush().unwrap();
    drop(db);
    let db = Database::open(directory.path()).unwrap();
    let phase = PcoVec::<Height, StoredU8, Budgeted>::import(&db, "phase", Version::ONE).unwrap();
    let position =
        PcoVec::<Height, StoredBool, Budgeted>::import(&db, "position", Version::ONE).unwrap();
    assert_eq!((phase.collect(), position.collect()), expected);
}

fn sources(source: &[impl ReadableVec<Height, Cents>; 5]) -> [&dyn ReadableVec<Height, Cents>; 5] {
    source
        .each_ref()
        .map(|s| s as &dyn ReadableVec<Height, Cents>)
}
