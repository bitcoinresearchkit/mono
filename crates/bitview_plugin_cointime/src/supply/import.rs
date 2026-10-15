use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyIndexedVec, LazySpotValuePerBlock};
use brk_types::{Cents, Height, Version};
use vecdb::ReadableBoxedVec;

use super::{super::activity, Vecs};

impl Vecs {
    pub(crate) fn new(
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        activity: &activity::Vecs,
        all_chain: &AllChainSources,
    ) -> Self {
        let vaulted = LazyIndexedVec::new(
            "vaulted_supply_sats_source",
            Version::ZERO,
            &activity.vaultedness.height,
            &all_chain.supply,
            |_, vaultedness, supply| supply * vaultedness,
        );
        let active = LazyIndexedVec::new(
            "active_supply_sats_source",
            Version::ZERO,
            &activity.liveliness.height,
            &all_chain.supply,
            |_, liveliness, supply| supply * liveliness,
        );

        let vaulted = LazySpotValuePerBlock::from_sats_source(
            "vaulted_supply",
            version,
            &vaulted,
            mappings,
            spot_price,
        );
        Self {
            hodled_or_lost: LazySpotValuePerBlock::identity(
                "hodled_or_lost_supply",
                version,
                &vaulted,
            ),
            vaulted,
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
