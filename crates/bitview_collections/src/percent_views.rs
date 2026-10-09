#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Fixed-point storage and its one public view, as a percentage.
#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
#[cfg_attr(feature = "storage", traversable(merge))]
pub struct PercentViews<A, C> {
    /// Fixed-point storage: parts per million (1,000,000 represents 1.0) or basis points.
    #[cfg_attr(feature = "storage", traversable(hidden))]
    pub fixed: A,
    /// As a percentage.
    pub percent: C,
}
