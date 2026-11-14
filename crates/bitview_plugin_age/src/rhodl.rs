use bitview_cohort::AgeRange;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Date, Ratio};
use bitview_vecs::{LazyIndexedVec, LazyPerBlock};
use brk_types::{Cents, Timestamp, Version};
use vecdb::Ident;

use crate::RangeVecs;

pub(crate) fn new(
    version: Version,
    ranges: &AgeRange<RangeVecs>,
    mappings: &MappingsVecs,
) -> LazyPerBlock<Ratio> {
    let old = LazyIndexedVec::new(
        "rhodl_old_capital_source",
        version,
        ranges._1y_to_18m.capital.cents_source(),
        ranges._18m_to_2y.capital.cents_source(),
        |_, first: Cents, second: Cents| first + second,
    );
    // Keep the division in f64 until the market-age weighting has been applied.
    let ratio = LazyIndexedVec::new(
        "rhodl_capital_ratio_source",
        version,
        ranges._1d_to_1w.capital.cents_source(),
        &old,
        |_, young: Cents, old: Cents| f64::from(young) / f64::from(old),
    );
    let genesis = Timestamp::from(Date::new(2009, 1, 3));
    let weighted = LazyIndexedVec::new(
        "rhodl_ratio_source",
        version,
        &ratio,
        &mappings.timestamp.monotonic,
        move |_, ratio, timestamp: Timestamp| Ratio::from(ratio * timestamp.days_since(genesis)),
    );
    LazyPerBlock::from_height_source::<Ident>("rhodl_ratio", version, &weighted, mappings)
}
