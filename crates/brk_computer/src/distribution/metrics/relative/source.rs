use brk_types::{PartsPerMillion64, PartsPerMillionSigned32};

use crate::{
    distribution::metrics::{
        RealizedSources, SupplySources, UnrealizedAggregateSources, UnrealizedSources,
    },
    internal::LazyRatioPerBlock,
};

pub(crate) struct RelativeSource<'a> {
    pub supply: SupplySources,
    pub unrealized: UnrealizedSources,
    pub unrealized_aggregate: UnrealizedAggregateSources,
    pub realized: RealizedSources,
    pub nupl: &'a LazyRatioPerBlock<PartsPerMillionSigned32, PartsPerMillion64>,
}
