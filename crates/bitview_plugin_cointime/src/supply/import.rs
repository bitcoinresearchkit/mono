use brk_error::Result;

use brk_types::{Cents, Height, Version};
use vecdb::{CachedBoxedVec, Database};

use super::{LazyBaseVecs, Vecs};
use bitview_compute::{LazySpotValuePerBlock, PerBlock};
use bitview_plugin_distribution::AllChainSources;

use super::super::activity;

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    spot_price: &CachedBoxedVec<Height, Cents>,
    activity: &activity::Vecs,
    all_chain: &AllChainSources,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, spot_price, activity, all_chain)
}

impl LazyBaseVecs {
    fn new(
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        activity: &activity::Vecs,
        all_chain: &AllChainSources,
    ) -> Self {
        let vaulted = all_chain.with_supply(
            "vaulted_supply_sats_source",
            Version::ZERO,
            &activity.vaultedness.height,
            |_, vaultedness, supply| supply * vaultedness,
        );
        let active = all_chain.with_supply(
            "active_supply_sats_source",
            Version::ZERO,
            &activity.liveliness.height,
            |_, liveliness, supply| supply * liveliness,
        );

        Self {
            vaulted: LazySpotValuePerBlock::from_sats_source(
                "vaulted_supply",
                version,
                vaulted,
                indexes,
                spot_price,
            ),
            active: LazySpotValuePerBlock::from_sats_source(
                "active_supply",
                version,
                active,
                indexes,
                spot_price,
            ),
        }
    }
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        activity: &activity::Vecs,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        Ok(Self {
            base: LazyBaseVecs::new(version, indexes, spot_price, activity, all_chain),
            active_supply_in_loss_share: PerBlock::forced_import(
                db,
                "cointime_supply_in_loss_share",
                version,
                indexes,
            )?,
        })
    }
}
