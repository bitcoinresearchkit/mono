use bitview_traversable::Traversable;

/// Lookback period days (includes 24h, unlike DCA)
pub const LOOKBACK_PERIOD_DAYS: ByLookbackPeriod<u32> = ByLookbackPeriod {
    _24h: 1,
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

/// Lookback period names
pub const LOOKBACK_PERIOD_NAMES: ByLookbackPeriod<&'static str> = ByLookbackPeriod {
    _24h: "24h",
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

/// Generic wrapper for lookback period-based data (includes 24h)
#[derive(Clone, Default, Traversable)]
pub struct ByLookbackPeriod<T> {
    pub _24h: T,
    pub _1w: T,
    pub _1m: T,
    pub _3m: T,
    pub _6m: T,
    pub _1y: T,
    pub _2y: T,
    pub _3y: T,
    pub _4y: T,
    pub _5y: T,
    pub _6y: T,
    pub _8y: T,
    pub _10y: T,
}

impl<T> ByLookbackPeriod<T> {
    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u32) -> Result<T, E>,
    {
        let n = LOOKBACK_PERIOD_NAMES;
        let d = LOOKBACK_PERIOD_DAYS;
        Ok(Self {
            _24h: create(n._24h, d._24h)?,
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

    pub fn try_from_period<U, F, E>(period: &ByLookbackPeriod<U>, mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u32, &U) -> Result<T, E>,
    {
        let n = LOOKBACK_PERIOD_NAMES;
        let d = LOOKBACK_PERIOD_DAYS;
        Ok(Self {
            _24h: create(n._24h, d._24h, &period._24h)?,
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
