use bitview_cohort::{AgeAggregate, AgeAggregateId, ProfitabilityRange, ProfitabilityRangeId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyRatioPerBlock, LazySpotValuePerBlockWithDeltas,
    LazyWindowStartVec, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, CentsSats, Height, PartsPerMillionSigned32, Sats, Version};
use vecdb::{AnyStoredVec, Database, PcoVecValue, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use crate::bucket::Bucket;

type Sources<T, M> = ProfitabilityRange<AgeAggregate<CachedSeries<Height, T, M>>>;

/// The same profitability metric layout for all seven age filters.
#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    pub supply: ProfitabilityRange<AgeAggregate<LazySpotValuePerBlockWithDeltas>>,
    pub realized_cap: ProfitabilityRange<AgeAggregate<LazyFiatPerBlock<Cents>>>,
    /// Absolute profit or loss, according to the represented profitability band.
    pub unrealized_pnl: ProfitabilityRange<AgeAggregate<LazyFiatPerBlock<Cents>>>,
    /// Signed net unrealized P&L divided by the cohort's own market cap.
    pub nupl: ProfitabilityRange<AgeAggregate<LazyRatioPerBlock<PartsPerMillionSigned32>>>,
    #[traversable(hidden)]
    supply_stored: Sources<Sats, M>,
    #[traversable(hidden)]
    cap_stored: Sources<Cents, M>,
    #[traversable(hidden)]
    pnl_stored: Sources<Cents, M>,
    #[traversable(hidden)]
    nupl_stored: Sources<PartsPerMillionSigned32, M>,
}

impl Metrics {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Box<Self>> {
        let supply_stored = Self::import_sources(db, "supply_sats", version)?;
        let cap_stored = Self::import_sources(db, "realized_cap_cents", version)?;
        let pnl_stored = Self::import_sources(db, "unrealized_pnl_cents", version)?;
        let nupl_stored = Self::import_sources(db, "nupl_ppm", version)?;
        let supply = Self::series(&supply_stored, "supply", |name, source| {
            LazySpotValuePerBlockWithDeltas::from_sats_source(
                name, version, source, mappings, windows, spot,
            )
        });
        let realized_cap = Self::series(&cap_stored, "realized_cap", |name, source| {
            LazyFiatPerBlock::from_cents_source(name, version, source, mappings)
        });
        let unrealized_pnl = Self::series(&pnl_stored, "unrealized_pnl", |name, source| {
            LazyFiatPerBlock::from_cents_source(name, version, source, mappings)
        });
        let nupl = Self::series(&nupl_stored, "nupl", |name, source| {
            LazyRatioPerBlock::from_height_source(name, version, source, mappings)
        });
        Ok(Box::new(Self {
            supply,
            realized_cap,
            unrealized_pnl,
            nupl,
            supply_stored,
            cap_stored,
            pnl_stored,
            nupl_stored,
        }))
    }

    fn import_sources<T: PcoVecValue>(
        db: &Database,
        metric: &str,
        version: Version,
    ) -> Result<Sources<T, Rw>> {
        ProfitabilityRange::try_from_fn(|band| {
            let name = band.select(ProfitabilityRange::names()).id;
            AgeAggregate::try_from_fn(|filter| {
                import_cached(db, &Self::metric_name(name, filter, metric), version)
            })
        })
    }

    fn series<T: PcoVecValue, S>(
        sources: &Sources<T, Rw>,
        metric: &str,
        mut build: impl FnMut(&str, &CachedSeries<Height, T>) -> S,
    ) -> ProfitabilityRange<AgeAggregate<S>> {
        ProfitabilityRange::from_fn(|band| {
            let name = band.select(ProfitabilityRange::names()).id;
            AgeAggregate::from_fn(|filter| {
                build(
                    &Self::metric_name(name, filter, metric),
                    filter.select(band.select(sources)),
                )
            })
        })
    }

    fn metric_name(band: &str, filter: AgeAggregateId, metric: &str) -> String {
        if filter == AgeAggregateId::All {
            format!("{band}_{metric}")
        } else {
            format!("{band}_{}_{metric}", filter.name())
        }
    }

    pub(crate) fn push(&mut self, spot: Cents, values: &ProfitabilityRange<Bucket>) {
        for &band in ProfitabilityRangeId::ALL {
            let bucket = band.select(values);
            let pnl = AgeAggregate::from_fn(|filter| {
                Self::pnl(
                    spot,
                    *filter.select(&bucket.cap),
                    *filter.select(&bucket.supply),
                    band.is_profit(),
                )
            });
            let nupl = AgeAggregate::from_fn(|filter| {
                Self::nupl(
                    spot,
                    *filter.select(&bucket.cap),
                    *filter.select(&bucket.supply),
                )
            });
            Self::push_filter(band.select_mut(&mut self.supply_stored), &bucket.supply);
            Self::push_filter(band.select_mut(&mut self.cap_stored), &bucket.cap);
            Self::push_filter(band.select_mut(&mut self.pnl_stored), &pnl);
            Self::push_filter(band.select_mut(&mut self.nupl_stored), &nupl);
        }
    }

    fn push_filter<T: PcoVecValue + Copy>(
        target: &mut AgeAggregate<CachedSeries<Height, T>>,
        values: &AgeAggregate<T>,
    ) {
        for (target, &value) in target.iter_mut().zip(values.iter()) {
            target.push(value);
        }
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply_stored
            .iter_mut()
            .flat_map(|v| v.iter_mut())
            .map(|v| v as &mut dyn AnyStoredVec)
            .chain(
                self.cap_stored
                    .iter_mut()
                    .flat_map(|v| v.iter_mut())
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain(
                self.pnl_stored
                    .iter_mut()
                    .flat_map(|v| v.iter_mut())
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
            .chain(
                self.nupl_stored
                    .iter_mut()
                    .flat_map(|v| v.iter_mut())
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
    }

    fn pnl(spot: Cents, cap: Cents, supply: Sats, profit: bool) -> Cents {
        let market = CentsSats::from_price_sats(spot, supply).to_cents_rounded();
        if profit {
            market.saturating_sub(cap)
        } else {
            cap.saturating_sub(market)
        }
    }

    fn nupl(spot: Cents, cap: Cents, supply: Sats) -> PartsPerMillionSigned32 {
        let spot = spot.as_u128();
        let supply = supply.as_u128();
        if spot == 0 || supply == 0 {
            PartsPerMillionSigned32::ZERO
        } else {
            let price = cap.as_u128() * Sats::ONE_BTC_U128 / supply;
            PartsPerMillionSigned32::from((spot as f64 - price as f64) / spot as f64)
        }
    }
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
