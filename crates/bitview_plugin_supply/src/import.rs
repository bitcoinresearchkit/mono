use bitview_collections::Windows;
use bitview_distribution::AllChainSources;
use bitview_plugin::ImportContext;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::{Halving, PartsPerMillionSigned64};
use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight,
    LazyWindowStartVec, LazyWindowVec,
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
        holders: &HoldersVecs,
        all_chain: &AllChainSources,
        transactions: &TransactionsVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();
        let supply_metrics = &holders.cohorts.all.supply.total;

        let circulating = LazyPerBlock::from_lazy::<Ident, Sats>(
            "circulating_supply",
            version,
            &supply_metrics.btc,
        );

        let burned = burned::Vecs::import(&db, version, mappings, window_starts)?;

        // Scheduled annual issuance (the block's scheduled subsidy x 52,560 blocks) over the
        // supply: the reciprocal of stock-to-flow.
        let inflation_version = version + Version::new(3);
        let inflation_source = all_chain.with_supply(
            "inflation_rate_ppm_source",
            inflation_version,
            &supply_metrics.sats.height,
            |height, _, supply| {
                PartsPerMillionSigned64::from(if supply <= Sats::FIFTY_BTC {
                    f64::NAN
                } else {
                    Halving::from(height).subsidy().as_u128() as f64 * 52_560.0
                        / supply.as_u128() as f64
                })
            },
        );
        let inflation_rate = LazyPercentPerBlock::from_height_source(
            "inflation_rate",
            inflation_version,
            &inflation_source,
            mappings,
        );

        // Velocity
        let velocity = velocity::Vecs::new(version, mappings, all_chain, transactions)?;

        let market_cap = LazyFiatPerBlock::from_cents_source(
            "market_cap",
            version,
            holders.all_market_cap(),
            mappings,
        );

        // Market cap delta (change + rate across 4 windows)
        let market_cap_delta = LazyRollingDeltasFiatFromHeight::new(
            "market_cap_delta",
            version + Version::new(4),
            &market_cap.cents.height,
            window_starts,
            mappings,
        );

        let growth_version = version + Version::new(3);
        let realized_cap = &holders.cohorts.all.realized.cap.cents.height;
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
                LazyPercentPerBlock::from_height_source(&name, growth_version, &source, mappings)
            });

        let this = Self {
            db,
            circulating,
            burned,
            inflation_rate,
            velocity,
            market_cap,
            market_cap_delta,
            market_minus_realized_cap_growth_rate,
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
