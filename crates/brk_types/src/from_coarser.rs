/// Maps a coarser-grained index to the last finer-grained index it covers
/// (e.g. a halving or difficulty epoch to its last height).
pub trait FromCoarserIndex<T>
where
    T: Ord + From<usize>,
{
    /// Returns the maximum fine-grained index represented by the coarse index.
    /// Note: May exceed actual data length - use `max_from` for bounded results.
    fn max_from_(coarser: T) -> usize;

    /// Returns the maximum fine-grained index, bounded by the data length.
    /// Returns 0 if len is 0 (empty data).
    fn max_from(coarser: T, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        Self::max_from_(coarser).min(len - 1)
    }
}
