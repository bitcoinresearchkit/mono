use bitview_cohort::{ADDR_TYPE_IDS, AddrTypeId};
use brk_types::{Cents, Height, Sats, Version};
use schemars::JsonSchema;
use vecdb::{
    CachedBoxedVec, CachedVec, PcoVec, PcoVecValue, ReadOnlyColumnarVec, ReadableColumnarVec,
    UnaryTransform,
};

use crate::{
    CACHE_BUDGET, CachedWindowStartVec, Identity, LazyColumnPerBlock,
    LazyColumnPerBlockCumulativeRolling, LazyColumnSpotValuePerBlock, LazyPerBlock,
    LazyPerBlockCumulativeAverage, LazyPerBlockCumulativeRolling, LazySpotValuePerBlock,
    NumericValue, Windows,
};

use super::WithAddrTypes;

impl<T> WithAddrTypes<LazyColumnPerBlock<T, AddrTypeId>, LazyPerBlock<T>>
where
    T: NumericValue + JsonSchema + 'static,
{
    pub fn from_columnar_source(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, AddrTypeId>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let all_source = CACHE_BUDGET.wrap(source.sum_columns(name, version, ADDR_TYPE_IDS));
        let all =
            LazyPerBlock::from_height_source::<Identity<T>>(name, version, all_source, indexes);
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            LazyColumnPerBlock::new(
                &format!("{type_name}_{name}"),
                version,
                source,
                column,
                indexes,
            )
        });

        Self { all, by_addr_type }
    }
}

impl WithAddrTypes<LazyColumnSpotValuePerBlock<AddrTypeId>, LazySpotValuePerBlock> {
    pub fn from_columnar_spot_value_source(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, Sats>, AddrTypeId>,
        indexes: &crate::IndexSources,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let sats =
            CachedVec::wrap(source.sum_columns(&format!("{name}_sats"), version, ADDR_TYPE_IDS));
        let all = LazySpotValuePerBlock::from_sats_source(name, version, sats, indexes, spot_price);
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            LazyColumnSpotValuePerBlock::new(
                &format!("{type_name}_{name}"),
                version,
                source,
                column,
                indexes,
                spot_price,
            )
        });

        Self { all, by_addr_type }
    }
}

impl<T>
    WithAddrTypes<
        LazyColumnPerBlockCumulativeRolling<T, AddrTypeId>,
        LazyPerBlockCumulativeRolling<T>,
    >
where
    T: NumericValue + JsonSchema + PcoVecValue,
{
    pub fn from_columnar_cumulative_source(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, AddrTypeId>,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative = CachedVec::wrap(source.sum_columns(
            &format!("{name}_cumulative"),
            version,
            ADDR_TYPE_IDS,
        ));
        let all = LazyPerBlockCumulativeRolling::from_cumulative_source(
            name,
            version,
            cumulative,
            cached_starts,
            indexes,
        );
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            LazyColumnPerBlockCumulativeRolling::new(
                &format!("{type_name}_{name}"),
                version,
                source,
                column,
                indexes,
                cached_starts,
            )
        });

        Self { all, by_addr_type }
    }
}

impl<T, C, F> WithAddrTypes<LazyPerBlockCumulativeAverage<T, C, F>>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema + PcoVecValue,
    F: UnaryTransform<C, T>,
{
    pub fn from_columnar_cumulative_average_source(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, C>, AddrTypeId>,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative_name = format!("{name}_cumulative");
        let cumulative = source.sum_columns(&cumulative_name, version, ADDR_TYPE_IDS);
        let all =
            LazyPerBlockCumulativeAverage::new(name, version, &cumulative, indexes, cached_starts);
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            let name = format!("{type_name}_{name}");
            let cumulative = source.column(&format!("{name}_cumulative"), version, column);
            LazyPerBlockCumulativeAverage::new(&name, version, &cumulative, indexes, cached_starts)
        });

        Self { all, by_addr_type }
    }
}
