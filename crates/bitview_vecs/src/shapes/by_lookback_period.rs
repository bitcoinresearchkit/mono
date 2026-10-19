use bitview_traversable::Traversable;

#[derive(Clone, Default, Traversable)]
pub struct ByLookbackPeriod<T> {
    /// Uses a trailing 24-hour monotonic-time window.
    _24h: T,
    /// Uses a trailing 7-day monotonic-time window.
    _1w: T,
    /// Uses a trailing 30-day monotonic-time window.
    _1m: T,
    /// Uses a trailing 90-day monotonic-time window.
    _3m: T,
    /// Uses a trailing 180-day monotonic-time window.
    _6m: T,
    /// Uses a trailing 365-day monotonic-time window.
    _1y: T,
    /// Uses a trailing 730-day monotonic-time window.
    _2y: T,
    /// Uses a trailing 1,095-day monotonic-time window.
    _3y: T,
    /// Uses a trailing 1,460-day monotonic-time window.
    _4y: T,
    /// Uses a trailing 1,825-day monotonic-time window.
    _5y: T,
    /// Uses a trailing 2,190-day monotonic-time window.
    _6y: T,
    /// Uses a trailing 2,920-day monotonic-time window.
    _8y: T,
    /// Uses a trailing 3,650-day monotonic-time window.
    _10y: T,
}

impl<T> ByLookbackPeriod<T> {
    /// Builds each period from its name and length in days.
    pub fn from_fn(mut f: impl FnMut(&'static str, usize) -> T) -> Self {
        Self {
            _24h: f("24h", 1),
            _1w: f("1w", 7),
            _1m: f("1m", 30),
            _3m: f("3m", 3 * 30),
            _6m: f("6m", 6 * 30),
            _1y: f("1y", 365),
            _2y: f("2y", 2 * 365),
            _3y: f("3y", 3 * 365),
            _4y: f("4y", 4 * 365),
            _5y: f("5y", 5 * 365),
            _6y: f("6y", 6 * 365),
            _8y: f("8y", 8 * 365),
            _10y: f("10y", 10 * 365),
        }
    }
}
