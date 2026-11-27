use bitview_distribution::AllChainSources;
use bitview_plugin::ImportContext;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::{Multiple, PartsPerMillionSigned64};
use bitview_vecs::{
    LazyFiatPerBlock, LazyIndexedVec, LazyPerBlock, LazyPercentPerBlock,
    LazyRollingDeltasFiatFromHeight, LazyWindowStartVec, PricedSum, ValuePerBlockCumulativeRolling,
    ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{BinaryTransform, Ident, LazyDeltaVec, LazyVec, ReadableCloneableVec};

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

        let inflation_version = version + Version::new(4);
        let inflation_source = LazyVec::new(
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

        let velocity = velocity::Vecs::new(version, mappings, all_chain, transactions);

        let market_cap = LazyFiatPerBlock::from_cents_source(
            "market_cap",
            version,
            holders.all_market_cap(),
            mappings,
        );

        let market_cap_delta = LazyRollingDeltasFiatFromHeight::new(
            "market_cap_delta",
            version + Version::new(4),
            &market_cap.cents.height,
            window_starts,
            mappings,
        );
        let nvt_source = LazyIndexedVec::new(
            "nvt_source",
            version,
            &all_chain.market_cap,
            &transactions.transfer_volume.sum._24h.cents.height,
            |_, cap, volume| Quotient::<Multiple>::apply(cap, volume),
        );
        let nvt = LazyPerBlock::from_height_source::<Ident>(
            "nvt",
            version,
            &nvt_source,
            mappings,
        );

        // Kalichkin's 90 days of volume: a dollar sum, none until the window is wholly priced.
        let volume_90d = LazyDeltaVec::<Height, Cents, Cents, PricedSum>::new(
            "transfer_volume_sum_90d_cents",
            version,
            transactions.transfer_volume.cumulative.cents.resolutions.height_source().read_only_boxed_clone(),
            mappings.lookback.start_vec(90).read_only_boxed_clone(),
        );
        let nvt_signal_source = LazyIndexedVec::new(
            "nvt_signal_source",
            version,
            &all_chain.market_cap,
            &volume_90d,
            |_, cap: Cents, volume: Cents| Multiple::from(f64::from(cap) * 90.0 / f64::from(volume)),
        );
        let nvt_signal = LazyPerBlock::from_height_source::<Ident>(
            "nvt_signal",
            version,
            &nvt_signal_source,
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
            nvt,
            nvt_signal,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
