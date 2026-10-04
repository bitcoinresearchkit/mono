#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// A fixed-point ratio with its decimal and percent views.
#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct FixedRatioViews<A, B, C> {
    /// Unitless ratio in parts per million; 1,000,000 represents 1.0.
    pub ppm: A,
    /// Unitless decimal ratio derived as parts per million divided by 1,000,000.
    pub ratio: B,
    /// Percentage derived as the decimal ratio multiplied by 100.
    pub percent: C,
}
