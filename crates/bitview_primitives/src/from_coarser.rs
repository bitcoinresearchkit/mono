/// Maps a coarser-grained index to the last finer-grained index it covers
/// (e.g. a halving or difficulty epoch to its last height).
pub trait FromCoarserIndex<T> {
    /// May exceed the data length; `max_from` caps it.
    fn max_from_unbounded(coarser: T) -> usize;

    /// Capped at `len - 1`; 0 when `len` is 0.
    fn max_from(coarser: T, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        Self::max_from_unbounded(coarser).min(len - 1)
    }
}
