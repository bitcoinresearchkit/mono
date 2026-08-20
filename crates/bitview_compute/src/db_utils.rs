use brk_error::Result;

use brk_types::Version;
use vecdb::AnyStoredVec;

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
