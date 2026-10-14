use bitview_traversable::Traversable;

/// Fixed-point storage and its one public view, as a percentage.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct PercentViews<A, C> {
    /// Fixed-point storage: parts per million (1,000,000 represents 1.0) or basis points.
    #[traversable(hidden)]
    pub fixed: A,
    /// As a percentage.
    pub(crate) percent: C,
}
