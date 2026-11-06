use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::LazySpotValuePerBlock;
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
        // Split once: active rounded, vaulted the rest, so the two always sum to the supply.
        let split = |name, complement: bool| {
            let weighted = if complement {
                LazySpotValuePerBlock::from_weighted_supply::<true>
            } else {
                LazySpotValuePerBlock::from_weighted_supply::<false>
            };
            weighted(
                name,
                version,
                &all_chain.supply,
                &activity.liveliness_source.height,
                mappings,
                spot_price,
            )
        };
        let vaulted = split("vaulted_supply", true);
        Self {
            hodled_or_lost: LazySpotValuePerBlock::identity(
                "hodled_or_lost_supply",
                version,
                &vaulted,
            ),
            vaulted,
            active: split("active_supply", false),
        }
    }
}
