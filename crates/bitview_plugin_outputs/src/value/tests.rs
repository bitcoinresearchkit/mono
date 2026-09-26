use bitview_vecs::CachedSeries;
use brk_exit::Exit;
use brk_types::{Height, OutputType, Sats, TxOutIndex, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, OverflowVec, PcoVec, ReadableVec, WritableVec,
};

use super::compute::compute_sats;

#[path = "../../../bitview_vecs/tests/common/cache.rs"]
mod cache;

#[test]
fn op_return_values_survive_resume_reorg_version_change_and_empty_chain() {
    cache::init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut first = PcoVec::<Height, TxOutIndex>::import(&db, "first", Version::ONE).unwrap();
    let mut types = BytesVec::<TxOutIndex, OutputType>::import(&db, "types", Version::ONE).unwrap();
    let mut values = OverflowVec::<TxOutIndex, Sats>::import(&db, "values", Version::ONE).unwrap();
    let mut target = CachedSeries::<Height, Sats>::import(&db, "total", Version::ONE).unwrap();
    let exit = Exit::new();
    let large = 5_000_000_000u64; // Exercises the overflow representation.
    for index in [0usize, 3, 3, 7] {
        first.push(TxOutIndex::from(index));
    }
    for (kind, sats) in [
        (OutputType::P2PKH, large),
        (OutputType::OpReturn, 0),
        (OutputType::OpReturn, 11),
        (OutputType::OpReturn, large),
        (OutputType::P2PKH, 17),
        (OutputType::OpReturn, 19),
        (OutputType::OpReturn, 23),
        (OutputType::P2PKH, large),
        (OutputType::P2PKH, 31),
    ] {
        types.push(kind);
        values.push(Sats::from(sats));
    }
    first.write().unwrap();
    types.write().unwrap();
    values.write().unwrap();
    let expected = [11, 11, large + 53, large + 53].map(Sats::from);
    compute_sats(&mut target, Height::ZERO, &first, &types, &values, &exit).unwrap();
    assert_eq!(target.collect(), expected);
    // Resume from an independently shortened output, then run with no new data.
    target.truncate_if_needed_at(2).unwrap();
    compute_sats(&mut target, Height::new(4), &first, &types, &values, &exit).unwrap();
    compute_sats(&mut target, Height::new(4), &first, &types, &values, &exit).unwrap();
    assert_eq!(target.collect(), expected);
    // Replace the suffix with a shorter fork. Empty blocks preserve the total.
    first.truncate_if_needed_at(3).unwrap();
    types.truncate_if_needed_at(3).unwrap();
    values.truncate_if_needed_at(3).unwrap();
    types.push(OutputType::OpReturn);
    values.push(Sats::from(29u64));
    first.write().unwrap();
    types.write().unwrap();
    values.write().unwrap();
    compute_sats(&mut target, Height::new(2), &first, &types, &values, &exit).unwrap();
    assert_eq!(target.collect(), [11u64, 11, 40].map(Sats::from));
    drop((first, types, values, target, db));

    let db = Database::open(directory.path()).unwrap();
    let mut first = PcoVec::<Height, TxOutIndex>::import(&db, "first", Version::ONE).unwrap();
    let types = BytesVec::<TxOutIndex, OutputType>::import(&db, "types", Version::ONE).unwrap();
    let values = OverflowVec::<TxOutIndex, Sats>::import(&db, "values", Version::ONE).unwrap();
    let mut target = CachedSeries::<Height, Sats>::import(&db, "total", Version::ONE).unwrap();
    assert_eq!(target.collect(), [11u64, 11, 40].map(Sats::from));
    // Version changes must invalidate an apparently complete target.
    drop(first);
    first = PcoVec::forced_import(&db, "first", Version::new(2)).unwrap();
    for index in [3usize, 4] {
        first.push(TxOutIndex::from(index));
    }
    first.write().unwrap();
    compute_sats(&mut target, Height::new(2), &first, &types, &values, &exit).unwrap();
    assert_eq!(target.collect(), [29u64, 29].map(Sats::from));
    first.truncate_if_needed_at(1).unwrap();
    first.write().unwrap();
    compute_sats(&mut target, Height::new(2), &first, &types, &values, &exit).unwrap();
    assert_eq!(target.collect(), [Sats::from(29u64)]);
    first.truncate_if_needed_at(0).unwrap();
    first.write().unwrap();
    compute_sats(&mut target, Height::new(2), &first, &types, &values, &exit).unwrap();
    drop((first, types, values, target, db));
    let db = Database::open(directory.path()).unwrap();
    let target = CachedSeries::<Height, Sats>::import(&db, "total", Version::ONE).unwrap();
    assert!(target.collect().is_empty());
}
