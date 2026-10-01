use brk_error::Result;
use brk_exit::Exit;
use brk_types::Version;
use vecdb::AnyStoredVec;

/// Resume a group at its shortest valid output, bounded by the requested rewind.
/// Every output is validated before the common rewind is persisted, including
/// updates with no new rows. Read-only views therefore observe the same prefix.
pub fn prepare_computed<'a, V: AnyStoredVec + ?Sized + 'a>(
    mut outputs: impl AsMut<[&'a mut V]>,
    version: Version,
    max_from: usize,
    exit: &Exit,
) -> Result<usize> {
    let _lock = exit.lock();
    let outputs = outputs.as_mut();
    let mut start = max_from;
    for vec in &mut *outputs {
        vec.any_validate_computed_version_or_reset(version)?;
        start = start.min(vec.len());
    }
    for vec in outputs {
        vec.any_truncate_if_needed_at(start)?;
        vec.write()?;
    }
    Ok(start)
}
