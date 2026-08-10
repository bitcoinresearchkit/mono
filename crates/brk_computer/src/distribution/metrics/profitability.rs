use brk_cohort::{
    Loss, PROFITABILITY_RANGE_COUNT, Profit, ProfitabilityId, ProfitabilityRange, ProfitabilityRow,
};
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, PartsPerMillionSigned32, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, CachedBoxedVec, ColumnId, ColumnarVec, Database, EagerVec, Exit,
    ImportableVec, PcoVec, ReadOnlyClone, ReadOnlyColumnarVec, Rw, StorageMode, WritableVec,
};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyColumnPerBlock, LazyColumnRatioPerBlock,
        LazyColumnSpotValuePerBlock, LazyColumnSpotValuePerBlockWithDeltas, Windows,
    },
    price,
};

#[derive(Clone, Traversable)]
pub struct WithSth<All, Sth = All> {
    pub all: All,
    pub sth: Sth,
}

#[derive(Clone, Traversable)]
pub struct ProfitabilityBucket {
    pub supply: WithSth<
        LazyColumnSpotValuePerBlockWithDeltas<ProfitabilityId>,
        LazyColumnSpotValuePerBlock<ProfitabilityId>,
    >,
    pub realized_cap: WithSth<
        LazyColumnPerBlock<Dollars, ProfitabilityId>,
        LazyColumnPerBlock<Dollars, ProfitabilityId>,
    >,
    pub unrealized_pnl: WithSth<
        LazyColumnPerBlock<Dollars, ProfitabilityId>,
        LazyColumnPerBlock<Dollars, ProfitabilityId>,
    >,
    pub nupl: LazyColumnRatioPerBlock<PartsPerMillionSigned32, ProfitabilityId>,
}

struct ProfitabilitySources {
    all_supply_sats: ReadOnlyColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>,
    sth_supply_sats: ReadOnlyColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>,
    all_realized_cap: ReadOnlyColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>,
    sth_realized_cap: ReadOnlyColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>,
    all_unrealized_pnl: ReadOnlyColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>,
    sth_unrealized_pnl: ReadOnlyColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>,
    nupl: ReadOnlyColumnarVec<PcoVec<Height, PartsPerMillionSigned32>, ProfitabilityId>,
}

impl ProfitabilityBucket {
    fn new(
        name: &str,
        version: Version,
        column: ProfitabilityId,
        sources: &ProfitabilitySources,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: WithSth {
                all: LazyColumnSpotValuePerBlockWithDeltas::new(
                    &format!("{name}_supply"),
                    version,
                    &sources.all_supply_sats,
                    column,
                    indexes,
                    cached_starts,
                    spot_price,
                ),
                sth: LazyColumnSpotValuePerBlock::new(
                    &format!("{name}_sth_supply"),
                    version,
                    &sources.sth_supply_sats,
                    column,
                    indexes,
                    spot_price,
                ),
            },
            realized_cap: WithSth {
                all: LazyColumnPerBlock::new(
                    &format!("{name}_realized_cap"),
                    version,
                    &sources.all_realized_cap,
                    column,
                    indexes,
                ),
                sth: LazyColumnPerBlock::new(
                    &format!("{name}_sth_realized_cap"),
                    version,
                    &sources.sth_realized_cap,
                    column,
                    indexes,
                ),
            },
            unrealized_pnl: WithSth {
                all: LazyColumnPerBlock::new(
                    &format!("{name}_unrealized_pnl"),
                    version,
                    &sources.all_unrealized_pnl,
                    column,
                    indexes,
                ),
                sth: LazyColumnPerBlock::new(
                    &format!("{name}_sth_unrealized_pnl"),
                    version,
                    &sources.sth_unrealized_pnl,
                    column,
                    indexes,
                ),
            },
            nupl: LazyColumnRatioPerBlock::new(
                &format!("{name}_nupl"),
                version + Version::new(4),
                &sources.nupl,
                column,
                indexes,
            ),
        }
    }
}

/// All profitability metrics: 25 ranges + 14 profit thresholds + 9 loss thresholds.
#[derive(Traversable)]
pub struct ProfitabilityMetrics<M: StorageMode = Rw> {
    pub range: ProfitabilityRange<ProfitabilityBucket>,
    pub profit: Profit<ProfitabilityBucket>,
    pub loss: Loss<ProfitabilityBucket>,
    pub all_supply_sats: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>>>,
    pub sth_supply_sats: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>>>,
    pub all_realized_cap:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>>,
    pub sth_realized_cap:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>>,
    pub all_unrealized_pnl:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>>,
    pub sth_unrealized_pnl:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>>,
    pub nupl:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, PartsPerMillionSigned32>, ProfitabilityId>>>,
}

impl<M: StorageMode> ProfitabilityMetrics<M> {
    pub fn iter(&self) -> impl Iterator<Item = &ProfitabilityBucket> {
        self.range
            .iter()
            .chain(self.profit.iter())
            .chain(self.loss.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ProfitabilityBucket> {
        self.range
            .iter_mut()
            .chain(self.profit.iter_mut())
            .chain(self.loss.iter_mut())
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.all_supply_sats.len().min(self.all_realized_cap.len())
    }
}

impl ProfitabilityMetrics {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let source_version = version + Version::TWO;
        let all_supply_sats =
            EagerVec::<ColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>>::forced_import(
                db,
                "profitability_all_supply_sats",
                source_version,
            )?;
        let sth_supply_sats =
            EagerVec::<ColumnarVec<PcoVec<Height, Sats>, ProfitabilityId>>::forced_import(
                db,
                "profitability_sth_supply_sats",
                source_version,
            )?;
        let all_realized_cap =
            EagerVec::<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>::forced_import(
                db,
                "profitability_all_realized_cap",
                source_version,
            )?;
        let sth_realized_cap =
            EagerVec::<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>::forced_import(
                db,
                "profitability_sth_realized_cap",
                source_version,
            )?;
        let all_unrealized_pnl =
            EagerVec::<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>::forced_import(
                db,
                "profitability_all_unrealized_pnl",
                source_version,
            )?;
        let sth_unrealized_pnl =
            EagerVec::<ColumnarVec<PcoVec<Height, Dollars>, ProfitabilityId>>::forced_import(
                db,
                "profitability_sth_unrealized_pnl",
                source_version,
            )?;
        let nupl = EagerVec::<ColumnarVec<
            PcoVec<Height, PartsPerMillionSigned32>,
            ProfitabilityId,
        >>::forced_import(
            db,
            "profitability_nupl_ppm",
            source_version + Version::new(4),
        )?;

        let sources = ProfitabilitySources {
            all_supply_sats: all_supply_sats.read_only_clone(),
            sth_supply_sats: sth_supply_sats.read_only_clone(),
            all_realized_cap: all_realized_cap.read_only_clone(),
            sth_realized_cap: sth_realized_cap.read_only_clone(),
            all_unrealized_pnl: all_unrealized_pnl.read_only_clone(),
            sth_unrealized_pnl: sth_unrealized_pnl.read_only_clone(),
            nupl: nupl.read_only_clone(),
        };

        let range = ProfitabilityId::range_series(|column, name| {
            ProfitabilityBucket::new(
                name,
                version,
                column,
                &sources,
                indexes,
                cached_starts,
                spot_price,
            )
        });

        let aggregate_version = version + Version::TWO;
        let profit = ProfitabilityId::profit_series(|column, name| {
            ProfitabilityBucket::new(
                name,
                aggregate_version,
                column,
                &sources,
                indexes,
                cached_starts,
                spot_price,
            )
        });

        let loss = ProfitabilityId::loss_series(|column, name| {
            ProfitabilityBucket::new(
                name,
                aggregate_version,
                column,
                &sources,
                indexes,
                cached_starts,
                spot_price,
            )
        });

        Ok(Self {
            range,
            profit,
            loss,
            all_supply_sats,
            sth_supply_sats,
            all_realized_cap,
            sth_realized_cap,
            all_unrealized_pnl,
            sth_unrealized_pnl,
            nupl,
        })
    }

    #[inline(always)]
    pub(crate) fn push_ranges(
        &mut self,
        all_supply_sats: [Sats; PROFITABILITY_RANGE_COUNT],
        sth_supply_sats: [Sats; PROFITABILITY_RANGE_COUNT],
        all_realized_cap: [Dollars; PROFITABILITY_RANGE_COUNT],
        sth_realized_cap: [Dollars; PROFITABILITY_RANGE_COUNT],
    ) {
        self.all_supply_sats
            .push(ProfitabilityRow::from_ranges(all_supply_sats));
        self.sth_supply_sats
            .push(ProfitabilityRow::from_ranges(sth_supply_sats));
        self.all_realized_cap
            .push(ProfitabilityRow::from_ranges(all_realized_cap));
        self.sth_realized_cap
            .push(ProfitabilityRow::from_ranges(sth_realized_cap));
    }

    pub(crate) fn compute(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            all_supply_sats,
            sth_supply_sats,
            all_realized_cap,
            sth_realized_cap,
            all_unrealized_pnl,
            sth_unrealized_pnl,
            nupl,
            ..
        } = self;
        let max_from = starting_lengths.height;

        all_unrealized_pnl.compute_transform3(
            max_from,
            &prices.spot.cents.height,
            all_realized_cap,
            all_supply_sats,
            |(height, spot, cap, supply, ..)| (height, unrealized_pnl_row(spot, cap, supply)),
            exit,
        )?;
        sth_unrealized_pnl.compute_transform3(
            max_from,
            &prices.spot.cents.height,
            sth_realized_cap,
            sth_supply_sats,
            |(height, spot, cap, supply, ..)| (height, unrealized_pnl_row(spot, cap, supply)),
            exit,
        )?;
        nupl.compute_transform3(
            max_from,
            &prices.spot.cents.height,
            all_realized_cap,
            all_supply_sats,
            |(height, spot, cap, supply, ..)| (height, nupl_row(spot, cap, supply)),
            exit,
        )?;

        Ok(())
    }

    pub(crate) fn collect_all_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 7] {
        [
            &mut self.all_supply_sats as &mut dyn AnyStoredVec,
            &mut self.sth_supply_sats,
            &mut self.all_realized_cap,
            &mut self.sth_realized_cap,
            &mut self.all_unrealized_pnl,
            &mut self.sth_unrealized_pnl,
            &mut self.nupl,
        ]
    }
}

fn unrealized_pnl_row(
    spot: Cents,
    cap: ProfitabilityRow<Dollars>,
    supply: ProfitabilityRow<Sats>,
) -> ProfitabilityRow<Dollars> {
    ProfitabilityId::from_fn(|column| {
        let market_value =
            f64::from(Dollars::from(spot)) * f64::from(Bitcoin::from(*column.get(&supply)));
        let realized_cap = f64::from(*column.get(&cap));
        let pnl = if column.is_profit() {
            market_value - realized_cap
        } else {
            realized_cap - market_value
        }
        .max(0.0);
        Dollars::from(pnl)
    })
}

fn nupl_row(
    spot: Cents,
    cap: ProfitabilityRow<Dollars>,
    supply: ProfitabilityRow<Sats>,
) -> ProfitabilityRow<PartsPerMillionSigned32> {
    ProfitabilityId::from_fn(|column| {
        let spot = spot.as_u128();
        let supply = column.get(&supply).as_u128();
        if spot == 0 || supply == 0 {
            PartsPerMillionSigned32::ZERO
        } else {
            let realized_price =
                Cents::from(*column.get(&cap)).as_u128() * Sats::ONE_BTC_U128 / supply;
            PartsPerMillionSigned32::from((spot as f64 - realized_price as f64) / spot as f64)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use brk_cohort::PROFIT_COUNT;

    #[test]
    fn expanded_profitability_thresholds_match_prefix_and_suffix_sums() {
        let ranges = std::array::from_fn(|index| Sats::from(index as u64 + 1));
        let row = ProfitabilityRow::from_ranges(ranges);
        let sum = |values: &[Sats]| {
            values
                .iter()
                .copied()
                .fold(Sats::default(), |total, value| total + value)
        };

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
        let supply = ProfitabilityId::from_fn(|_| Sats::ONE_BTC);
        let cap = ProfitabilityId::from_fn(|column| {
            Dollars::from(if column.is_profit() { 1.0 } else { 3.0 })
        });
        let spot = Cents::from(200_u64);

        let pnl = unrealized_pnl_row(spot, cap.clone(), supply.clone());
        let nupl = nupl_row(spot, cap, supply);

        for column in ProfitabilityId::ALL {
            assert_eq!(*column.get(&pnl), Dollars::from(1.0));
            assert_eq!(
                *column.get(&nupl),
                PartsPerMillionSigned32::from(if column.is_profit() { 0.5 } else { -0.5 })
            );
        }
    }
}
