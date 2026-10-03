use bitview_plugin_distribution_common::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{BoundedRatioPerBlock, LazySpotValuePerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{super::activity, LazyBaseVecs, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        activity: &activity::Vecs,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        Ok(Vecs {
            base: LazyBaseVecs::new(version, mappings, spot_price, activity, all_chain),
            active_supply_in_loss_share: BoundedRatioPerBlock::forced_import(
                db,
                "cointime_supply_in_loss_share",
                version + Version::ONE,
                mappings,
            )?,
        })
    }
}

impl LazyBaseVecs {
    fn new(
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
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
                &vaulted,
                mappings,
                spot_price,
            ),
            active: LazySpotValuePerBlock::from_sats_source(
                "active_supply",
                version,
                &active,
                mappings,
                spot_price,
            ),
        }
    }
}
