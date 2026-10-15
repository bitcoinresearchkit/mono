use bitview_distribution::AllChainSources;
use bitview_plugin::ImportContext;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight,
    LazyWindowStartVec, ValuePerBlockCumulativeRolling, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Ident, LazyVec, ReadableCloneableVec};

use crate::{STORAGE, Vecs, velocity};

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

        let burned = ValuePerBlockCumulativeRolling::import(
            &db,
            "burned",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;

        // Scheduled annual issuance (the block's scheduled subsidy x 52,560 blocks) over the
        // supply; undefined only without supply (block 0: the genesis output is unspendable).
        let inflation_version = version + Version::new(4);
        let inflation_source = LazyVec::init(
            "inflation_rate_ppm_source",
            inflation_version,
            supply_metrics.sats.height.read_only_boxed_clone(),
            |height: Height, supply: Sats| {
                PartsPerMillionSigned64::from(if supply.is_zero() {
                    f64::NAN
                } else {
                    height.subsidy().as_u128() as f64 * 52_560.0 / supply.as_u128() as f64
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
        let velocity = velocity::Vecs::new(version, mappings, all_chain, transactions);

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

        let this = Self {
            db,
            circulating,
            burned,
            inflation_rate,
            velocity,
            market_cap,
            market_cap_delta,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
