use brk_error::Result;
use brk_types::Version;
use vecdb::{Budgeted, Database, ImportableVec, PcoVec, PcoVecValue, Rw, StorageMode, VecIndex};

/// One stored source and its shared, budgeted cache.
pub type CachedSeries<I, T, M = Rw> = <M as StorageMode>::Stored<PcoVec<I, T, Budgeted>>;

pub fn import_cached<I: VecIndex, T: PcoVecValue>(
    db: &Database,
    name: &str,
    version: Version,
) -> Result<CachedSeries<I, T>> {
    Ok(PcoVec::import(db, name, version)?)
}
