use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PriceRatio;
use bitview_vecs::{LazyPerBlock, LazyRatioPerBlock, PriceWithMvrv, PriceWithRatio, import_cached};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, CentsSigned, Dollars, Height, Version};
use vecdb::{Database, Ident, ReadableCloneableVec};

use super::{Vecs, aviv_nupl::AvivNupl};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        all_chain: &AllChainSources,
        cointime_cap: &impl ReadableCloneableVec<Height, Cents>,
        active_cap: &impl ReadableCloneableVec<Height, Dollars>,
        investor_cap: &impl ReadableCloneableVec<Height, CentsSigned>,
    ) -> Result<Self> {
        let cointime_source = all_chain.with_supply(
            "cointime_price_cents_source",
            version,
            cointime_cap,
            |_, cap, supply| Cents::from(f64::from(cap) / f64::from(Bitcoin::from(supply))),
        );
        let vaulted_cents = import_cached(db, "vaulted_price_cents", version)?;
        let active_cents = import_cached(db, "active_price_cents", version)?;
        // ONE: null without a positive investor capitalization.
        let true_market_mean_cents =
            import_cached(db, "true_market_mean_cents", version + Version::ONE)?;
        let true_market_mean = PriceWithRatio::import(
            db,
            "true_market_mean",
            version,
            &true_market_mean_cents,
            mappings,
        )?;
        Ok(Vecs {
            vaulted: PriceWithMvrv::import(
                db,
                "vaulted_price",
                "vaulted_mvrv",
                version,
                &vaulted_cents,
                mappings,
            )?,
            active: PriceWithMvrv::import(
                db,
                "active_price",
                "active_mvrv",
                version,
                &active_cents,
                mappings,
            )?,
            aviv: LazyPerBlock::from_lazy::<Ident, PriceRatio>(
                "aviv",
                version,
                &true_market_mean.relative.ratio,
            ),
            aviv_nupl: LazyRatioPerBlock::from_ratio::<Dollars, CentsSigned, AvivNupl>(
                "aviv_nupl",
                version,
                active_cap,
                investor_cap,
                mappings,
            ),
            true_market_mean,
            cointime: PriceWithMvrv::import(
                db,
                "cointime_price",
                "cointime_mvrv",
                version,
                &cointime_source,
                mappings,
            )?,
            vaulted_cents,
            active_cents,
            true_market_mean_cents,
        })
    }
}
