use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_distribution_common::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_vecs::{
    LazyFiatPerBlock, LazyFixedRatioPerBlock, LazyPerBlock, LazyRollingDeltasFiatFromHeight,
    LazySpotValuePerBlock, LazyValuePerBlock, LazyWindowStartVec, LazyWindowVec,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Ident, ReadableCloneableVec, ReadableVec};

use crate::{STORAGE, Vecs, burned, velocity};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        distribution_aggregated: &AggregatedVecs,
        cointime: &CointimeVecs,
        all_chain: &AllChainSources,
        transactions: &TransactionsVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();
        let supply_metrics = &distribution_aggregated.cohorts.all.supply.total;

        let circulating =
            LazyValuePerBlock::spot_identity("circulating_supply", supply_metrics, version);

        let burned = burned::Vecs::import(&db, version, mappings)?;

        let inflation_version = version + Version::TWO;
        let inflation_source = LazyWindowVec::<Height, Sats, PartsPerMillionSigned64>::new(
            "inflation_rate_ppm_source",
            inflation_version,
            &supply_metrics.sats.height,
            window_starts._1y,
            false,
            |current, previous, _| {
                if previous <= Sats::FIFTY_BTC {
                    PartsPerMillionSigned64::from(f64::NAN)
                } else {
                    PartsPerMillionSigned64::from(f64::from(current) / f64::from(previous) - 1.0)
                }
            },
        );
        let inflation_rate = LazyFixedRatioPerBlock::from_height_source(
            "inflation_rate",
            inflation_version,
            &inflation_source,
            mappings,
        );

        // Velocity
        let velocity = velocity::Vecs::new(version, mappings, all_chain, transactions)?;

        // Market cap - lazy fiat (cents + usd) from distribution supply
        let market_cap = LazyFiatPerBlock::from_lazy("market_cap", version, &supply_metrics.cents);

        // Market cap delta (change + rate across 4 windows)
        let market_cap_delta = LazyRollingDeltasFiatFromHeight::new(
            "market_cap_delta",
            version + Version::new(4),
            &market_cap.cents.height,
            window_starts,
            mappings,
        );

        let growth_version = version + Version::new(3);
        let realized_cap = &distribution_aggregated
            .cohorts
            .all
            .realized
            .cap
            .cents
            .height;
        let market_minus_realized_cap_growth_rate =
            window_starts.map_with_suffix(|suffix, starts| {
                let name = format!("market_minus_realized_cap_growth_rate_{suffix}");
                let source = Self::market_minus_realized_cap_growth(
                    all_chain,
                    &format!("{name}_source"),
                    growth_version,
                    realized_cap,
                    starts.read_only_boxed_clone(),
                );
                LazyPerBlock::from_height_source::<Ident>(&name, growth_version, &source, mappings)
            });

        let hodled_or_lost = LazySpotValuePerBlock::identity(
            "hodled_or_lost_supply",
            version,
            &cointime.supply.vaulted,
        );

        let this = Self {
            db,
            circulating,
            burned,
            inflation_rate,
            velocity,
            market_cap,
            market_cap_delta,
            market_minus_realized_cap_growth_rate,
            hodled_or_lost,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    fn market_minus_realized_cap_growth(
        all_chain: &AllChainSources,
        name: &str,
        version: Version,
        realized_cap: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: impl ReadableVec<Height, Height> + Clone + 'static,
    ) -> LazyWindowVec<Height, (Cents, Cents), PartsPerMillionSigned64> {
        let caps = all_chain.with_market_cap(
            &format!("{name}_caps"),
            Version::ZERO,
            realized_cap,
            |_, realized, market| (realized, market),
        );

        LazyWindowVec::new(
            name,
            version,
            &caps,
            &window_starts,
            false,
            |current, previous, _| {
                let growth = |current: Cents, previous: Cents| {
                    if previous == Cents::ZERO {
                        0.0
                    } else {
                        (f64::from(current) - f64::from(previous)) / f64::from(previous)
                    }
                };
                PartsPerMillionSigned64::from(
                    growth(current.1, previous.1) - growth(current.0, previous.0),
                )
            },
        )
    }
}
