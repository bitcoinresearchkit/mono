use brk_error::Result;
use brk_types::Version;
use vecdb::AnyStoredVec;

/// Resume a group at its shortest valid output, bounded by the requested rewind.
/// Every output is validated before the group is truncated to the common start.
pub fn prepare_computed<'a, V: AnyStoredVec + ?Sized + 'a>(
    mut outputs: impl AsMut<[&'a mut V]>,
    version: Version,
    max_from: usize,
) -> Result<usize> {
    let outputs = outputs.as_mut();
    let mut start = max_from;
    for vec in &mut *outputs {
        vec.any_validate_computed_version_or_reset(version)?;
        start = start.min(vec.len());
    }
    for vec in outputs {
        vec.any_truncate_if_needed_at(start)?;
    }
    Ok(start)
}
