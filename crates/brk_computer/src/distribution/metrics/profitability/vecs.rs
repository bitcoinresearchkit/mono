use std::ops::AddAssign;

use brk_cohort::{
    ByTerm, ProfitabilityId, ProfitabilityRange, ProfitabilityRow, UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, PartsPerMillionSigned32, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, CachedBoxedVec, Database, PcoVec, PcoVecValue, ReadOnlyColumnarVec,
    ReadableBoxedVec, Rw, StorageMode,
};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlock, Identity, LazyColumnRatioPerBlock, LazyPerBlock,
        LazySpotValuePerBlockWithDeltas, Windows,
    },
};

use super::{
    TermProfitabilityRangeId,
    compute::{nupl_row, sum_terms, unrealized_pnl_rows},
};

const VERSION: Version = Version::new(7);

#[derive(Traversable)]
pub struct ProfitabilityVecs<M: StorageMode = Rw> {
    pub supply: ColumnarPerBlock<
        Sats,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazySpotValuePerBlockWithDeltas>>,
        M,
    >,
    pub realized_cap: ColumnarPerBlock<
        Dollars,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazyPerBlock<Dollars>>>,
        M,
    >,
    pub unrealized_pnl: ColumnarPerBlock<
        Dollars,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazyPerBlock<Dollars>>>,
        M,
    >,
    pub nupl: ColumnarPerBlock<
        PartsPerMillionSigned32,
        ProfitabilityId,
        ProfitabilityRow<LazyColumnRatioPerBlock<PartsPerMillionSigned32, ProfitabilityId>>,
        M,
    >,
}

impl<M: StorageMode> ProfitabilityVecs<M> {
    pub(crate) fn min_stateful_len(&self) -> usize {
        self.supply
            .height
            .len()
            .min(self.realized_cap.height.len())
            .min(self.unrealized_pnl.height.len())
            .min(self.nupl.height.len())
    }
}

impl ProfitabilityVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + VERSION;
        let supply = ColumnarPerBlock::forced_import(
            db,
            "profitability_supply_sats_by_term_and_range",
            version,
            |source| {
                Self::series(source, "supply", version, |name, source| {
                    LazySpotValuePerBlockWithDeltas::from_boxed_sats_source(
                        name,
                        version,
                        source,
                        indexes,
                        cached_starts,
                        spot_price,
                    )
                })
            },
        )?;
        let realized_cap = ColumnarPerBlock::forced_import(
            db,
            "profitability_realized_cap_by_term_and_range",
            version,
            |source| {
                Self::series(source, "realized_cap", version, |name, source| {
                    LazyPerBlock::from_uncached_boxed_height_source::<Identity<Dollars>>(
                        name, version, source, indexes,
                    )
                })
            },
        )?;
        let unrealized_pnl = ColumnarPerBlock::forced_import(
            db,
            "profitability_unrealized_pnl_by_term_and_range",
            version,
            |source| {
                Self::series(source, "unrealized_pnl", version, |name, source| {
                    LazyPerBlock::from_uncached_boxed_height_source::<Identity<Dollars>>(
                        name, version, source, indexes,
                    )
                })
            },
        )?;
        let nupl =
            ColumnarPerBlock::forced_import(db, "profitability_nupl_ppm", version, |source| {
                ProfitabilityId::series(|column, name| {
                    LazyColumnRatioPerBlock::new(
                        &format!("{name}_nupl"),
                        version,
                        source,
                        column,
                        indexes,
                    )
                })
            })?;

        Ok(Self {
            supply,
            realized_cap,
            unrealized_pnl,
            nupl,
        })
    }

    fn series<T, S>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, TermProfitabilityRangeId>,
        metric: &str,
        version: Version,
        mut build: impl FnMut(&str, ReadableBoxedVec<Height, T>) -> S,
    ) -> ProfitabilityRow<UTXOAggregate<S>>
    where
        T: PcoVecValue + AddAssign,
    {
        ProfitabilityId::series(|column, cohort_name| {
            UTXOAggregate::from_fn(|aggregate| {
                let name = Self::metric_name(cohort_name, aggregate, metric);
                let source = TermProfitabilityRangeId::source(
                    source,
                    &format!("{name}_source"),
                    version,
                    aggregate,
                    column.ranges(),
                );
                build(&name, source)
            })
        })
    }

    fn metric_name(cohort: &str, aggregate: UTXOAggregateId, metric: &str) -> String {
        match aggregate {
            UTXOAggregateId::All => format!("{cohort}_{metric}"),
            UTXOAggregateId::Sth | UTXOAggregateId::Lth => {
                format!("{cohort}_{}_{metric}", aggregate.cohort_name().id)
            }
        }
    }

    #[inline(always)]
    pub(crate) fn push(
        &mut self,
        spot: Cents,
        supply: ByTerm<ProfitabilityRange<Sats>>,
        realized_cap: ByTerm<ProfitabilityRange<Dollars>>,
    ) {
        let all_supply = sum_terms(&supply);
        let all_realized_cap = sum_terms(&realized_cap);
        let unrealized_pnl = unrealized_pnl_rows(spot, &realized_cap, &supply);
        let nupl = nupl_row(spot, &all_realized_cap, &all_supply);

        self.supply.push(supply);
        self.realized_cap.push(realized_cap);
        self.unrealized_pnl.push(unrealized_pnl);
        self.nupl.push(nupl);
    }

    pub(crate) fn collect_all_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 4] {
        [
            self.supply.stored_mut(),
            self.realized_cap.stored_mut(),
            self.unrealized_pnl.stored_mut(),
            self.nupl.stored_mut(),
        ]
    }
}
