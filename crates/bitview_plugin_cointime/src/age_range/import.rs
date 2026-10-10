use bitview_cohort::{AgeRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{BoundedOdds, BoundedToRatio};
use bitview_vecs::{
    LazyPerBlock, LazyPerBlockCumulativeRolling, LazySpotValuePerBlock, LazyWindowStartVec,
    PerBlockCumulativeRolling, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{RangeVecs, SideVecs, Vecs};

const VERSION: Version = Version::new(4);

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        parent_version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        age: &AgeVecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let ranges = AgeRange::try_from_fn(|id| -> Result<_> {
            let name = CohortContext::Utxo.full_name(id.cohort());
            let name = |metric: &str| format!("{name}_{metric}");
            let flow = |metric: &str| {
                PerBlockCumulativeRolling::import(
                    db,
                    &name(metric),
                    version,
                    mappings,
                    window_starts,
                )
            };
            let wakefulness_source =
                import_cached(db, &name("wakefulness_bounded_source"), version)?;
            let supply = &id.select(&age.ranges).supply.total.stored;
            Ok(RangeVecs {
                coindays_created: LazyPerBlockCumulativeRolling::from_cumulative_source(
                    &name("coindays_created"),
                    version,
                    id.select(&age.ranges).coindays_created.cumulative_source(),
                    window_starts,
                    mappings,
                ),
                coindays_consumed: flow("coindays_consumed")?,
                coindays_stored: flow("coindays_stored")?,
                wakefulness: LazyPerBlock::from_height_source::<BoundedToRatio>(
                    &name("wakefulness"),
                    version,
                    &wakefulness_source,
                    mappings,
                ),
                awake_to_dormant: LazyPerBlock::from_height_source::<BoundedOdds>(
                    &name("awake_to_dormant"),
                    version,
                    &wakefulness_source,
                    mappings,
                ),
                awake: SideVecs {
                    supply: LazySpotValuePerBlock::from_weighted_supply::<false>(
                        &name("awake_supply"),
                        version,
                        supply,
                        &wakefulness_source,
                        mappings,
                        spot_price,
                    ),
                },
                dormant: SideVecs {
                    supply: LazySpotValuePerBlock::from_weighted_supply::<true>(
                        &name("dormant_supply"),
                        version,
                        supply,
                        &wakefulness_source,
                        mappings,
                        spot_price,
                    ),
                },
                wakefulness_source,
            })
        })?;
        Ok(Vecs { ranges })
    }
}
