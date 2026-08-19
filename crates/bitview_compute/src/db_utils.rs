use brk_error::Result;

use std::path::Path;

use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PAGE_SIZE};

pub fn open_db(parent_path: &Path, db_name: &str, page_multiplier: usize) -> Result<Database> {
    let db = Database::open(&parent_path.join(db_name))?;
    db.set_min_len(PAGE_SIZE * page_multiplier)?;
    Ok(db)
}

pub fn finalize_db(db: &Database, traversable: &impl Traversable) -> Result<()> {
    db.retain_regions(
        traversable
            .iter_any_exportable()
            .flat_map(|v| v.region_names())
            .collect(),
    )?;
    db.compact()?;
    Ok(())
}

pub fn validate_any_computed_version_or_reset(
    vec: &mut dyn AnyStoredVec,
    dependency_version: Version,
) -> Result<()> {
    let computed_version = vec.header().vec_version() + dependency_version;
    if computed_version != vec.header().computed_version() {
        vec.mut_header().update_computed_version(computed_version);
        if !vec.is_empty() {
            vec.any_reset()?;
        }
    }
    Ok(())
}
