use brk_types::{PartsPerMillionSigned32, PriceRatio};

use crate::metrics::{
    RealizedSources, SupplySources, UnrealizedAggregateSources, UnrealizedSources,
};
use bitview_vecs::LazyRatioPerBlock;

pub struct RelativeSource<'a> {
    pub supply: SupplySources,
    pub unrealized: UnrealizedSources,
    pub unrealized_aggregate: UnrealizedAggregateSources,
    pub realized: RealizedSources,
    pub nupl: &'a LazyRatioPerBlock<PartsPerMillionSigned32, PriceRatio>,
}
