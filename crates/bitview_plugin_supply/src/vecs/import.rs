use brk_error::Result;

use std::path::Path;

use brk_types::{Cents, Height, PartsPerMillionSigned64, Sats, Version};
use vecdb::{CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec};

use crate::burned;
use bitview_compute::{
    CACHE_BUDGET, CachedWindowStartVec, Identity, LazyFiatPerBlock, LazyPerBlock,
    LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight, LazySpotValuePerBlock, LazyValuePerBlock,
    LazyWindowVec, Windows,
    db_utils::{finalize_db, open_db},
};
use bitview_plugin_distribution::AllChainSources;

use super::Vecs;

const VERSION: Version = Version::ONE;

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub fn forced_import(
        parent: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        distribution: &bitview_plugin_distribution::Vecs,
        cointime: &bitview_plugin_cointime::Vecs,
        all_chain: &AllChainSources,
        transactions: &bitview_plugin_transactions::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent, crate::DB_NAME, 1_000_000)?;

        let version = parent_version + VERSION;
        let supply_metrics = &distribution.cohorts.supply.total.cohorts.all;

        let circulating =
            LazyValuePerBlock::spot_identity("circulating_supply", supply_metrics, version);

        let burned = burned::forced_import(&db, version, indexes)?;

        let inflation_version = version + Version::TWO;
        let inflation_source = LazyWindowVec::<Height, Sats, PartsPerMillionSigned64>::new(
            "inflation_rate_ppm_source",
            inflation_version,
            supply_metrics.sats.height.read_only_boxed_clone(),
            cached_starts._1y.read_only_cached_boxed_clone(),
            false,
            |current, previous, _| {
                if previous <= Sats::FIFTY_BTC {
                    PartsPerMillionSigned64::from(f64::NAN)
                } else {
                    PartsPerMillionSigned64::from(f64::from(current) / f64::from(previous) - 1.0)
                }
            },
        );
        let inflation_source = CACHE_BUDGET.wrap(inflation_source);
        let inflation_rate = LazyPercentPerBlock::from_height_source(
            "inflation_rate",
            inflation_version,
            inflation_source,
            indexes,
        );

        // Velocity
        let velocity = crate::velocity::forced_import(version, indexes, all_chain, transactions)?;

        // Market cap - lazy fiat (cents + usd) from distribution supply
        let market_cap = LazyFiatPerBlock::from_lazy("market_cap", version, &supply_metrics.cents);

        // Market cap delta (change + rate across 4 windows)
        let market_cap_delta = LazyRollingDeltasFiatFromHeight::new(
            "market_cap_delta",
            version + Version::new(4),
            &market_cap.cents.height,
            cached_starts,
            indexes,
        );

        let growth_version = version + Version::new(3);
        let realized_cap = &distribution.cohorts.realized.cap.cohorts.all.cents.height;
        let market_minus_realized_cap_growth_rate =
            cached_starts.map_with_suffix(|suffix, starts| {
                let name = format!("market_minus_realized_cap_growth_rate_{suffix}");
                let source = Self::market_minus_realized_cap_growth(
                    all_chain,
                    &format!("{name}_source"),
                    growth_version,
                    realized_cap,
                    starts.read_only_cached_boxed_clone(),
                );
                LazyPerBlock::from_height_source::<Identity<PartsPerMillionSigned64>>(
                    &name,
                    growth_version,
                    source,
                    indexes,
                )
            });

        let hodled_or_lost = LazySpotValuePerBlock::identity(
            "hodled_or_lost_supply",
            version,
            &cointime.supply.vaulted,
        );

        let this = Self {
            plugin_gate: Default::default(),
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
        finalize_db(&this.db, &this)?;
        Ok(this)
    }

    fn market_minus_realized_cap_growth(
        all_chain: &AllChainSources,
        name: &str,
        version: Version,
        realized_cap: &(impl ReadableCloneableVec<Height, Cents> + 'static),
        window_starts: CachedBoxedVec<Height, Height>,
    ) -> impl TypedVec<I = Height, T = PartsPerMillionSigned64>
    + ReadableVec<Height, PartsPerMillionSigned64>
    + Clone
    + 'static {
        let caps = all_chain.with_market_cap(
            &format!("{name}_caps"),
            Version::ZERO,
            realized_cap,
            |_, realized, market| (realized, market),
        );

        LazyWindowVec::new(
            name,
            version,
            caps.read_only_boxed_clone(),
            window_starts,
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
