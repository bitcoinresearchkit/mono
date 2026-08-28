use bitview_traversable::Traversable;
use brk_types::PercentileId;

#[derive(Clone, Traversable)]
pub struct PercentilePrices<T> {
    /// Uses the 5th percentile.
    pub pct05: T,
    /// Uses the 10th percentile.
    pub pct10: T,
    /// Uses the 15th percentile.
    pub pct15: T,
    /// Uses the 20th percentile.
    pub pct20: T,
    /// Uses the 25th percentile.
    pub pct25: T,
    /// Uses the 30th percentile.
    pub pct30: T,
    /// Uses the 35th percentile.
    pub pct35: T,
    /// Uses the 40th percentile.
    pub pct40: T,
    /// Uses the 45th percentile.
    pub pct45: T,
    /// Uses the 50th percentile.
    pub pct50: T,
    /// Uses the 55th percentile.
    pub pct55: T,
    /// Uses the 60th percentile.
    pub pct60: T,
    /// Uses the 65th percentile.
    pub pct65: T,
    /// Uses the 70th percentile.
    pub pct70: T,
    /// Uses the 75th percentile.
    pub pct75: T,
    /// Uses the 80th percentile.
    pub pct80: T,
    /// Uses the 85th percentile.
    pub pct85: T,
    /// Uses the 90th percentile.
    pub pct90: T,
    /// Uses the 95th percentile.
    pub pct95: T,
}

impl<T> PercentilePrices<T> {
    pub fn from_fn(mut f: impl FnMut(PercentileId) -> T) -> Self {
        Self {
            pct05: f(PercentileId::Pct05),
            pct10: f(PercentileId::Pct10),
            pct15: f(PercentileId::Pct15),
            pct20: f(PercentileId::Pct20),
            pct25: f(PercentileId::Pct25),
            pct30: f(PercentileId::Pct30),
            pct35: f(PercentileId::Pct35),
            pct40: f(PercentileId::Pct40),
            pct45: f(PercentileId::Pct45),
            pct50: f(PercentileId::Pct50),
            pct55: f(PercentileId::Pct55),
            pct60: f(PercentileId::Pct60),
            pct65: f(PercentileId::Pct65),
            pct70: f(PercentileId::Pct70),
            pct75: f(PercentileId::Pct75),
            pct80: f(PercentileId::Pct80),
            pct85: f(PercentileId::Pct85),
            pct90: f(PercentileId::Pct90),
            pct95: f(PercentileId::Pct95),
        }
    }
}
