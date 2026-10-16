/// How a window view ([`LazyDeltaVec`](super::LazyDeltaVec)) combines the current value with the
/// value before the window.
///
/// `S` is the source type read from the vec, `T` is the output type produced.
/// When `S = T` (e.g., rolling sums), the operation is same-type.
/// When `S != T` (e.g., delta change/rate), the operation converts between types.
pub trait DeltaOp<S, T>: Send + Sync + 'static {
    /// The number of elements the window covers, `[start, h]`.
    #[inline]
    fn count(h: usize, start: usize) -> usize {
        h - start + 1
    }

    fn combine(current: S, before: S, count: usize) -> T;
}
