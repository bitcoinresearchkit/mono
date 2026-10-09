use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PriceRatio;
use bitview_transforms::MvrvToNupl;
use bitview_vecs::{
    LazyPerBlock, LazyPriceWithRatioPerBlock, LazyRatioPerBlock, PriceWithRatioPerBlock,
};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Height, Version};
use vecdb::{Database, Ident, ReadableBoxedVec, ReadableCloneableVec};

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

        let vaulted: PriceWithRatioPerBlock = import!("vaulted_price");
        let active: PriceWithRatioPerBlock = import!("active_price");
        let true_market_mean: PriceWithRatioPerBlock = import!("true_market_mean");
        let cointime = LazyPriceWithRatioPerBlock::from_height_source(
            "cointime_price",
            version,
            &cointime_source,
            mappings,
            spot_price,
        );
        let jargon =
            |name: &str, ratio| LazyPerBlock::from_lazy::<Ident, PriceRatio>(name, version, ratio);
        Ok(Vecs {
            vaulted_mvrv: jargon("vaulted_mvrv", &vaulted.relative.ratio),
            active_mvrv: jargon("active_mvrv", &active.relative.ratio),
            aviv: jargon("aviv", &true_market_mean.relative.ratio),
            aviv_nupl: LazyRatioPerBlock::from_lazy_source::<MvrvToNupl, PriceRatio>(
                "aviv_nupl",
                version,
                &true_market_mean.relative.fixed,
            ),
            cointime_mvrv: jargon("cointime_mvrv", &cointime.relative.ratio),
            vaulted,
            active,
            true_market_mean,
            cointime,
        })
    }
}
