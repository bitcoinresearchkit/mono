use brk_error::Result;

use std::ops::{Add, AddAssign};

use bitview_traversable::Traversable;
use brk_cohort::{
    ByTerm, ProfitabilityId, ProfitabilityRange, ProfitabilityRangeId, ProfitabilityRow,
    UTXOAggregate, UTXOAggregateId,
};
use brk_types::{Cents, CentsSats, Height, PartsPerMillionSigned32, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, CachedBoxedVec, ColumnId, Database, PcoVec, PcoVecValue,
    ReadOnlyColumnarVec, ReadableBoxedVec, Rw, StorageMode,
};

use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlock, LazyColumnRatioPerBlock, LazyFiatPerBlock,
    LazySpotValuePerBlockWithDeltas, Windows,
};

use super::TermProfitabilityRangeId;

const VERSION: Version = Version::new(8);

#[derive(Traversable)]
pub struct ProfitabilityVecs<M: StorageMode = Rw> {
    /// Unspent supply grouped by short- or long-term ownership and by the
    /// output's percentage profit or loss relative to spot price.
    pub supply: ColumnarPerBlock<
        Sats,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazySpotValuePerBlockWithDeltas>>,
        M,
    >,
    /// Creation-date value of unspent supply grouped by short- or long-term
    /// ownership and by percentage profit or loss relative to spot price.
    pub realized_cap: ColumnarPerBlock<
        Cents,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazyFiatPerBlock<Cents>>>,
        M,
    >,
    /// Absolute unrealized profit or loss of unspent supply grouped by short-
    /// or long-term ownership and by percentage profit or loss relative to
    /// spot price.
    pub unrealized_pnl: ColumnarPerBlock<
        Cents,
        TermProfitabilityRangeId,
        ProfitabilityRow<UTXOAggregate<LazyFiatPerBlock<Cents>>>,
        M,
    >,
    /// Net unrealized profit/loss as a share of market cap for the selected
    /// profitability range: spot price minus aggregate realized price, divided
    /// by spot price.
    pub nupl: ColumnarPerBlock<
        PartsPerMillionSigned32,
        ProfitabilityId,
        ProfitabilityRow<LazyColumnRatioPerBlock<PartsPerMillionSigned32, ProfitabilityId>>,
        M,
    >,
}

impl<M: StorageMode> ProfitabilityVecs<M> {
    pub fn min_resume_len(&self) -> usize {
        self.supply
            .height
            .len()
            .min(self.realized_cap.height.len())
            .min(self.unrealized_pnl.height.len())
            .min(self.nupl.height.len())
    }
}

impl ProfitabilityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
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
                    LazyFiatPerBlock::from_boxed_cents_source(name, version, source, indexes)
                })
            },
        )?;
        let unrealized_pnl = ColumnarPerBlock::forced_import(
            db,
            "profitability_unrealized_pnl_by_term_and_range",
            version,
            |source| {
                Self::series(source, "unrealized_pnl", version, |name, source| {
                    LazyFiatPerBlock::from_boxed_cents_source(name, version, source, indexes)
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
    pub fn push(
        &mut self,
        spot: Cents,
        supply: ByTerm<ProfitabilityRange<Sats>>,
        realized_cap: ByTerm<ProfitabilityRange<Cents>>,
    ) {
        let all_supply = Self::sum_terms(&supply);
        let all_realized_cap = Self::sum_terms(&realized_cap);
        let unrealized_pnl = Self::unrealized_pnl_rows(spot, &realized_cap, &supply);
        let nupl = Self::nupl_row(spot, &all_realized_cap, &all_supply);

        self.supply.push(supply);
        self.realized_cap.push(realized_cap);
        self.unrealized_pnl.push(unrealized_pnl);
        self.nupl.push(nupl);
    }

    pub fn collect_all_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 4] {
        [
            self.supply.stored_mut(),
            self.realized_cap.stored_mut(),
            self.unrealized_pnl.stored_mut(),
            self.nupl.stored_mut(),
        ]
    }

    fn sum_terms<T>(rows: &ByTerm<ProfitabilityRange<T>>) -> ProfitabilityRange<T>
    where
        T: Add<Output = T> + Copy,
    {
        ProfitabilityRange::from_fn(|range| *range.select(&rows.short) + *range.select(&rows.long))
    }

    fn unrealized_pnl_rows(
        spot: Cents,
        cap: &ByTerm<ProfitabilityRange<Cents>>,
        supply: &ByTerm<ProfitabilityRange<Sats>>,
    ) -> ByTerm<ProfitabilityRange<Cents>> {
        ByTerm {
            short: Self::unrealized_pnl_row(spot, &cap.short, &supply.short),
            long: Self::unrealized_pnl_row(spot, &cap.long, &supply.long),
        }
    }

    fn unrealized_pnl_row(
        spot: Cents,
        cap: &ProfitabilityRange<Cents>,
        supply: &ProfitabilityRange<Sats>,
    ) -> ProfitabilityRange<Cents> {
        ProfitabilityRangeId::from_fn(|column| {
            let market_value =
                CentsSats::from_price_sats(spot, *column.get(supply)).to_cents_rounded();
            let realized_cap = *column.get(cap);
            if column.is_profit() {
                market_value.saturating_sub(realized_cap)
            } else {
                realized_cap.saturating_sub(market_value)
            }
        })
    }

    fn nupl_row(
        spot: Cents,
        cap: &ProfitabilityRange<Cents>,
        supply: &ProfitabilityRange<Sats>,
    ) -> ProfitabilityRow<PartsPerMillionSigned32> {
        let cap = ProfitabilityRow::from_ranges(cap.clone());
        let supply = ProfitabilityRow::from_ranges(supply.clone());
        ProfitabilityId::from_fn(|column| {
            let spot = spot.as_u128();
            let supply = column.get(&supply).as_u128();
            if spot == 0 || supply == 0 {
                PartsPerMillionSigned32::ZERO
            } else {
                let realized_price = column.get(&cap).as_u128() * Sats::ONE_BTC_U128 / supply;
                PartsPerMillionSigned32::from((spot as f64 - realized_price as f64) / spot as f64)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use brk_cohort::{
        ByTerm, PROFIT_COUNT, ProfitabilityId, ProfitabilityRangeId, ProfitabilityRow,
    };
    use brk_types::{Cents, PartsPerMillionSigned32, Sats};
    use vecdb::ColumnId;

    use super::ProfitabilityVecs;

    #[test]
    fn expanded_thresholds_match_prefix_and_suffix_sums() {
        let ranges = ProfitabilityRangeId::from_fn(|id| Sats::from(id.index() as u64 + 1));
        let row = ProfitabilityRow::from_ranges(ranges.clone());
        let sum = |values: &[Sats]| {
            values
                .iter()
                .copied()
                .fold(Sats::default(), |total, value| total + value)
        };

        let ranges: Vec<_> = ranges.iter().copied().collect();
        for (threshold, &column) in ProfitabilityId::profit_ids().iter().enumerate() {
            assert_eq!(
                *column.get(&row),
                sum(&ranges[..PROFIT_COUNT + 1 - threshold])
            );
        }
        for (threshold, &column) in ProfitabilityId::loss_ids().iter().enumerate() {
            assert_eq!(
                *column.get(&row),
                sum(&ranges[PROFIT_COUNT + 1 + threshold..])
            );
        }
    }

    #[test]
    fn derived_rows_preserve_profit_and_loss_polarity() {
        let supply = ProfitabilityRangeId::from_fn(|_| Sats::ONE_BTC);
        let cap = ProfitabilityRangeId::from_fn(|column| {
            Cents::from(if column.is_profit() { 100_u64 } else { 300_u64 })
        });
        let spot = Cents::from(200_u64);

        let cap = ByTerm {
            short: cap.clone(),
            long: cap.clone(),
        };
        let supply = ByTerm {
            short: supply.clone(),
            long: supply.clone(),
        };
        let pnl = ProfitabilityVecs::unrealized_pnl_rows(spot, &cap, &supply);
        let all_cap = ProfitabilityVecs::sum_terms(&cap);
        let all_supply = ProfitabilityVecs::sum_terms(&supply);
        let nupl = ProfitabilityVecs::nupl_row(spot, &all_cap, &all_supply);

        for column in ProfitabilityRangeId::ALL {
            assert_eq!(*column.get(&pnl.short), Cents::from(100_u64));
            assert_eq!(*column.get(&pnl.long), Cents::from(100_u64));
        }
        for column in ProfitabilityId::ALL {
            assert_eq!(
                *column.get(&nupl),
                PartsPerMillionSigned32::from(if column.is_profit() { 0.5 } else { -0.5 })
            );
        }
    }
}
