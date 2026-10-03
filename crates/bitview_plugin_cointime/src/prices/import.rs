use bitview_plugin_distribution_common::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPriceWithRatioPerBlock, PriceWithRatioPerBlock};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableCloneableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_chain: &AllChainSources,
        cointime_cap: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        macro_rules! import {
            ($name:expr) => {
                PriceWithRatioPerBlock::import(db, $name, version, mappings, spot_price)?
            };
        }

        let cointime_source = all_chain.with_supply(
            "cointime_price_cents_source",
            version,
            cointime_cap,
            |_, cap, supply| Cents::from(f64::from(cap) / f64::from(Bitcoin::from(supply))),
        );

        Ok(Vecs {
            vaulted: import!("vaulted_price"),
            active: import!("active_price"),
            true_market_mean: import!("true_market_mean"),
            cointime: LazyPriceWithRatioPerBlock::from_height_source(
                "cointime_price",
                version,
                &cointime_source,
                mappings,
                spot_price,
            ),
        })
    }
}
