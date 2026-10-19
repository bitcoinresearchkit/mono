/// How a window view ([`LazyDeltaVec`](super::LazyDeltaVec)) combines the current value with the
/// value before the window.
///
/// `S` is the source type read from the vec, `T` the output type produced.
pub trait DeltaOp<S, T>: Send + Sync + 'static {
    /// The count passed to `combine`; by default the window length `h - start + 1`.
    #[inline]
    fn count(h: usize, start: usize) -> usize {
        h - start + 1
    }

    fn combine(current: S, before: S, count: usize) -> T;
}
