use brk_error::Result;

use brk_types::{Bitcoin, Cents, Height, Version};
use vecdb::{CachedBoxedVec, Database, ReadableCloneableVec};

use super::Vecs;
use bitview_compute::{LazyPriceWithRatioPerBlock, PriceWithRatioPerBlock};
use bitview_plugin_distribution::AllChainSources;

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    spot_price: &CachedBoxedVec<Height, Cents>,
    all_chain: &AllChainSources,
    cointime_cap: &(impl ReadableCloneableVec<Height, Cents> + 'static),
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, spot_price, all_chain, cointime_cap)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_chain: &AllChainSources,
        cointime_cap: &(impl ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Result<Self> {
        macro_rules! import {
            ($name:expr) => {
                PriceWithRatioPerBlock::forced_import(db, $name, version, mappings, spot_price)?
            };
        }

        let cointime_source = all_chain.with_supply(
            "cointime_price_cents_source",
            version,
            cointime_cap,
            |_, cap, supply| Cents::from(f64::from(cap) / f64::from(Bitcoin::from(supply))),
        );

        Ok(Self {
            vaulted: import!("vaulted_price"),
            active: import!("active_price"),
            true_market_mean: import!("true_market_mean"),
            cointime: LazyPriceWithRatioPerBlock::from_height_source(
                "cointime_price",
                version,
                cointime_source,
                mappings,
                spot_price,
            ),
        })
    }
}
