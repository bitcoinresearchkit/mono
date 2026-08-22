use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct EmaVecs<T> {
    /// Uses a trailing 7-day span.
    pub _1w: T,
    /// Uses a trailing 8-day span.
    pub _8d: T,
    /// Uses a trailing 12-day span.
    pub _12d: T,
    /// Uses a trailing 13-day span.
    pub _13d: T,
    /// Uses a trailing 21-day span.
    pub _21d: T,
    /// Uses a trailing 26-day span.
    pub _26d: T,
    /// Uses a trailing 30-day span.
    pub _1m: T,
    /// Uses a trailing 34-day span.
    pub _34d: T,
    /// Uses a trailing 55-day span.
    pub _55d: T,
    /// Uses a trailing 89-day span.
    pub _89d: T,
    /// Uses a trailing 144-day span.
    pub _144d: T,
    /// Uses a trailing 200-day span.
    pub _200d: T,
    /// Uses a trailing 365-day span.
    pub _1y: T,
    /// Uses a trailing 730-day span.
    pub _2y: T,
    /// Uses a trailing 1,400-day span.
    pub _200w: T,
    /// Uses a trailing 1,460-day span.
    pub _4y: T,
}
