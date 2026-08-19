use bitview_traversable::Traversable;

use crate::ByDcaPeriod;

/// DCA CAGR period days (only periods at least two years long).
pub const DCA_CAGR_DAYS: ByDcaCagr<u32> = ByDcaCagr {
    _2y: 2 * 365,
    _3y: 3 * 365,
    _4y: 4 * 365,
    _5y: 5 * 365,
    _6y: 6 * 365,
    _8y: 8 * 365,
    _10y: 10 * 365,
};

/// DCA CAGR period names.
pub const DCA_CAGR_NAMES: ByDcaCagr<&'static str> = ByDcaCagr {
    _2y: "2y",
    _3y: "3y",
    _4y: "4y",
    _5y: "5y",
    _6y: "6y",
    _8y: "8y",
    _10y: "10y",
};

/// Generic wrapper for DCA CAGR data (periods at least two years long).
#[derive(Clone, Default, Traversable)]
pub struct ByDcaCagr<T> {
    pub _2y: T,
    pub _3y: T,
    pub _4y: T,
    pub _5y: T,
    pub _6y: T,
    pub _8y: T,
    pub _10y: T,
}

impl<T> ByDcaCagr<T> {
    pub fn try_new<U, F, E>(period: &ByDcaPeriod<U>, mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u32, &U) -> Result<T, E>,
    {
        let n = DCA_CAGR_NAMES;
        let d = DCA_CAGR_DAYS;
        Ok(Self {
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
