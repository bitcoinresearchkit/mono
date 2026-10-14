#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Open, high, low and close fields.
#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct Ohlc<T> {
    /// Opening price for each supported period. A populated period uses its
    /// first block-level price; an empty period carries the previous close.
    pub open: T,
    /// Highest price for each supported period. A populated period uses its
    /// maximum block-level price; an empty period carries the previous close.
    pub high: T,
    /// Lowest price for each supported period. A populated period uses its
    /// minimum block-level price; an empty period carries the previous close.
    pub low: T,
    /// Closing price for each supported period. A populated period uses its
    /// final block-level price; an empty period carries the previous close.
    pub close: T,
}
