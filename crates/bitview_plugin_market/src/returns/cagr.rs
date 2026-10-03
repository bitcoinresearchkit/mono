use bitview_collections::{ByLookbackPeriod, LOOKBACK_PERIOD_DAYS, LOOKBACK_PERIOD_NAMES};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPercentPerBlock;
use brk_types::{PartsPerMillionSigned64, Version};

/// Annualized spot-price returns over trailing periods of at least two years.
#[derive(Clone, Traversable)]
pub struct Cagr {
    /// Uses a trailing 730-day period.
    pub _2y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 1,095-day period.
    pub _3y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 1,460-day period.
    pub _4y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 1,825-day period.
    pub _5y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 2,190-day period.
    pub _6y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 2,920-day period.
    pub _8y: LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// Uses a trailing 3,650-day period.
    pub _10y: LazyPercentPerBlock<PartsPerMillionSigned64>,
}

impl Cagr {
    pub(super) fn new(
        version: Version,
        periods: &ByLookbackPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    ) -> Self {
        let create = |name, days: u32, source| {
            LazyPercentPerBlock::from_lazy_cagr(
                &format!("price_cagr_{name}"),
                version,
                (days / 365) as u8,
                source,
            )
        };
        let n = LOOKBACK_PERIOD_NAMES;
        let d = LOOKBACK_PERIOD_DAYS;
        Self {
            _2y: create(n._2y, d._2y, &periods._2y),
            _3y: create(n._3y, d._3y, &periods._3y),
            _4y: create(n._4y, d._4y, &periods._4y),
            _5y: create(n._5y, d._5y, &periods._5y),
            _6y: create(n._6y, d._6y, &periods._6y),
            _8y: create(n._8y, d._8y, &periods._8y),
            _10y: create(n._10y, d._10y, &periods._10y),
        }
    }
}
