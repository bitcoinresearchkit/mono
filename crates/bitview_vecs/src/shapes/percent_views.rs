use bitview_traversable::Traversable;

/// A fixed-point value and its public percentage view.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct PercentViews<A, C> {
    /// Fixed-point value in parts per million; 1,000,000 represents 1.0.
    #[traversable(hidden)]
    pub fixed: A,
    /// As a percentage.
    pub(crate) percent: C,
}
