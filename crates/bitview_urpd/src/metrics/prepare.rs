use brk_error::Result;
use brk_types::Version;
use vecdb::AnyStoredVec;

pub(super) fn prepare<'a>(
    vecs: impl Iterator<Item = &'a mut dyn AnyStoredVec>,
    version: Version,
    from: usize,
    end: usize,
) -> Result<usize> {
    let mut vecs = vecs.collect::<Vec<_>>();
    let mut start = from.min(end);
    for vec in &mut vecs {
        vec.any_validate_computed_version_or_reset(version)?;
        start = start.min(vec.len());
    }
    for vec in &mut vecs {
        vec.any_truncate_if_needed_at(start)?;
    }
    Ok(start)
}
