use bitview_traversable::Traversable;

use crate::ByLookbackPeriod;

/// DCA period identifiers with their day counts
pub const DCA_PERIOD_DAYS: ByDcaPeriod<u32> = ByDcaPeriod {
    _1w: 7,
    _1m: 30,
    _3m: 3 * 30,
    _6m: 6 * 30,
    _1y: 365,
    _2y: 2 * 365,
    _3y: 3 * 365,
    _4y: 4 * 365,
    _5y: 5 * 365,
    _6y: 6 * 365,
    _8y: 8 * 365,
    _10y: 10 * 365,
};

/// DCA period names
pub const DCA_PERIOD_NAMES: ByDcaPeriod<&'static str> = ByDcaPeriod {
    _1w: "1w",
    _1m: "1m",
    _3m: "3m",
    _6m: "6m",
    _1y: "1y",
    _2y: "2y",
    _3y: "3y",
    _4y: "4y",
    _5y: "5y",
    _6y: "6y",
    _8y: "8y",
    _10y: "10y",
};

/// Generic wrapper for DCA period-based data
#[derive(Clone, Default, Traversable)]
pub struct ByDcaPeriod<T> {
    /// Uses a trailing 7-day investment period.
    pub _1w: T,
    /// Uses a trailing 30-day investment period.
    pub _1m: T,
    /// Uses a trailing 90-day investment period.
    pub _3m: T,
    /// Uses a trailing 180-day investment period.
    pub _6m: T,
    /// Uses a trailing 365-day investment period.
    pub _1y: T,
    /// Uses a trailing 730-day investment period.
    pub _2y: T,
    /// Uses a trailing 1,095-day investment period.
    pub _3y: T,
    /// Uses a trailing 1,460-day investment period.
    pub _4y: T,
    /// Uses a trailing 1,825-day investment period.
    pub _5y: T,
    /// Uses a trailing 2,190-day investment period.
    pub _6y: T,
    /// Uses a trailing 2,920-day investment period.
    pub _8y: T,
    /// Uses a trailing 3,650-day investment period.
    pub _10y: T,
}

impl<T> ByDcaPeriod<T> {
    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u32) -> Result<T, E>,
    {
        let n = DCA_PERIOD_NAMES;
        let d = DCA_PERIOD_DAYS;
        Ok(Self {
            _1w: create(n._1w, d._1w)?,
            _1m: create(n._1m, d._1m)?,
            _3m: create(n._3m, d._3m)?,
            _6m: create(n._6m, d._6m)?,
            _1y: create(n._1y, d._1y)?,
            _2y: create(n._2y, d._2y)?,
            _3y: create(n._3y, d._3y)?,
            _4y: create(n._4y, d._4y)?,
            _5y: create(n._5y, d._5y)?,
            _6y: create(n._6y, d._6y)?,
            _8y: create(n._8y, d._8y)?,
            _10y: create(n._10y, d._10y)?,
        })
    }

    pub fn try_from_period<U, F, E>(period: &ByDcaPeriod<U>, mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u32, &U) -> Result<T, E>,
    {
        let n = DCA_PERIOD_NAMES;
        let d = DCA_PERIOD_DAYS;
        Ok(Self {
            _1w: create(n._1w, d._1w, &period._1w)?,
            _1m: create(n._1m, d._1m, &period._1m)?,
            _3m: create(n._3m, d._3m, &period._3m)?,
            _6m: create(n._6m, d._6m, &period._6m)?,
            _1y: create(n._1y, d._1y, &period._1y)?,
            _2y: create(n._2y, d._2y, &period._2y)?,
            _3y: create(n._3y, d._3y, &period._3y)?,
            _4y: create(n._4y, d._4y, &period._4y)?,
            _5y: create(n._5y, d._5y, &period._5y)?,
            _6y: create(n._6y, d._6y, &period._6y)?,
            _8y: create(n._8y, d._8y, &period._8y)?,
            _10y: create(n._10y, d._10y, &period._10y)?,
        })
    }
}

impl<T> ByDcaPeriod<&T> {
    /// Get the DCA-matching subset from lookback (excludes 24h)
    pub fn from_lookback(lookback: &ByLookbackPeriod<T>) -> ByDcaPeriod<&T> {
        ByDcaPeriod {
            _1w: &lookback._1w,
            _1m: &lookback._1m,
            _3m: &lookback._3m,
            _6m: &lookback._6m,
            _1y: &lookback._1y,
            _2y: &lookback._2y,
            _3y: &lookback._3y,
            _4y: &lookback._4y,
            _5y: &lookback._5y,
            _6y: &lookback._6y,
            _8y: &lookback._8y,
            _10y: &lookback._10y,
        }
    }
}
