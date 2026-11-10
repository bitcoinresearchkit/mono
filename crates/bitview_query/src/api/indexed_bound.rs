use crate::Result;

/// Which crossing to find in an ascending indexed source.
#[derive(Clone, Copy)]
pub(super) enum IndexedBound {
    AtLeast,
    Above,
}

/// The first index whose value is >= / > `target`, or `len` if none. The reader
/// must use one pinned publication and the source must be nondecreasing.
pub(super) fn indexed_bound<T: Ord>(
    mut end: usize,
    target: T,
    bound: IndexedBound,
    mut read: impl FnMut(usize) -> Result<T>,
) -> Result<usize> {
    let mut begin = 0;
    while begin < end {
        let mid = begin + (end - begin) / 2;
        let value = read(mid)?;
        if value < target || (matches!(bound, IndexedBound::Above) && value == target) {
            begin = mid + 1;
        } else {
            end = mid;
        }
    }
    Ok(begin)
}
