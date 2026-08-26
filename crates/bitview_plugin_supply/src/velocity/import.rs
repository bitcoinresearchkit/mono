use brk_error::Result;

use brk_types::{Cents, StoredF64, Version};

use super::Vecs;
use bitview_compute::{Identity, LazyPerBlock};
use bitview_plugin_distribution::AllChainSources;

impl Vecs {
    pub fn forced_import(
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        all_chain: &AllChainSources,
        transactions: &bitview_plugin_transactions::Vecs,
    ) -> Result<Self> {
        let volume = &transactions.volume.transfer_volume.sum._1y;
        let native_source = all_chain.with_supply(
            "velocity_btc_source",
            version,
            &volume.sats.height,
            |_, volume, supply| Self::ratio(f64::from(volume), f64::from(supply)),
        );
        let fiat_source = all_chain.with_market_cap(
            "velocity_usd_source",
            version,
            &volume.cents.height,
            |_, volume: Cents, market_cap| Self::ratio(f64::from(volume), f64::from(market_cap)),
        );

        Ok(Self {
            native: LazyPerBlock::from_height_source::<Identity<StoredF64>>(
                "velocity_btc",
                version,
                native_source,
                mappings,
            ),
            fiat: LazyPerBlock::from_height_source::<Identity<StoredF64>>(
                "velocity_usd",
                version,
                fiat_source,
                mappings,
            ),
        })
    }

    fn ratio(numerator: f64, denominator: f64) -> StoredF64 {
        if denominator != 0.0 {
            StoredF64::from(numerator / denominator)
        } else {
            StoredF64::from(0.0)
        }
    }
}
