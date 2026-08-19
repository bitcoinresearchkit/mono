use bitview_traversable::Traversable;
use brk_types::{Cents, Day1, Dollars, SatsFract, Version};
use vecdb::{ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use bitview_compute::{
    CentsUnsignedToDollars, DailyMappings, DollarsToSatsFract, LazyColumnDailyMetric,
    LazyDailyMetric,
};

#[derive(Clone, Traversable)]
pub struct LazyColumnPrice<C>
where
    C: ColumnId,
{
    /// Reported in USD per BTC.
    pub usd: LazyDailyMetric<Dollars, Cents>,
    /// Reported in cents per BTC.
    pub cents: LazyColumnDailyMetric<Cents, C>,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: LazyDailyMetric<SatsFract, Dollars>,
}

impl<C> LazyColumnPrice<C>
where
    C: ColumnId,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Day1, Cents>, C>,
        column: C,
        mappings: &DailyMappings,
    ) -> Self {
        let cents =
            LazyColumnDailyMetric::new(&format!("{name}_cents"), version, source, column, mappings);
        let usd = LazyDailyMetric::from_source::<CentsUnsignedToDollars>(
            name,
            version,
            cents.day1.read_only_boxed_clone(),
            mappings,
        );
        let sats = LazyDailyMetric::from_source::<DollarsToSatsFract>(
            &format!("{name}_sats"),
            version,
            usd.day1.read_only_boxed_clone(),
            mappings,
        );

        Self { usd, cents, sats }
    }
}
