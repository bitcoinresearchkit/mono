use bitview_cohort::{AgeRange, AgeRangeId, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{BoundedOdds, BoundedToRatio};
use bitview_vecs::{
    LazyPerBlock, LazySpotValuePerBlock, LazyWindowStartVec, PerBlockCumulativeRolling,
    import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{ActivitySeries, SupplyVecs, Vecs};

const VERSION: Version = Version::new(4);

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        parent_version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        distribution_age: &AgeVecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let import_coindays = |metric: &str| {
            AgeRange::try_from_fn(|id| {
                let name = format!("{}_{metric}", CohortContext::Utxo.full_name(id.cohort()));
                PerBlockCumulativeRolling::import(db, &name, version, mappings, window_starts)
            })
        };
        let coindays_consumed = import_coindays("coindays_consumed")?;
        let coindays_stored = import_coindays("coindays_stored")?;
        let activity_sources = AgeRange::try_from_fn(|id| {
            let name = format!(
                "{}_wakefulness_bounded_source",
                CohortContext::Utxo.full_name(id.cohort())
            );
            import_cached(db, &name, version)
        })?;
        let activity = ActivitySeries {
            wakefulness: AgeRangeId::series(CohortContext::Utxo, |id, name| {
                LazyPerBlock::from_height_source::<BoundedToRatio>(
                    &format!("{name}_wakefulness"),
                    version,
                    id.select(&activity_sources),
                    mappings,
                )
            }),
            dormancy: AgeRangeId::series(CohortContext::Utxo, |id, name| {
                LazyPerBlock::from_height_source::<BoundedToRatio<true>>(
                    &format!("{name}_dormancy"),
                    version,
                    id.select(&activity_sources),
                    mappings,
                )
            }),
            wakefulness_to_dormancy: AgeRangeId::series(CohortContext::Utxo, |id, name| {
                LazyPerBlock::from_height_source::<BoundedOdds>(
                    &format!("{name}_wakefulness_to_dormancy"),
                    version,
                    id.select(&activity_sources),
                    mappings,
                )
            }),
        };
        let import_supply = |side: &str, complement: bool| {
            AgeRangeId::series(CohortContext::Utxo, |id, name| {
                let name = format!("{name}_{side}_supply");
                let supply = distribution_age
                    .cohorts
                    .supply
                    .total
                    .stored
                    .get(id.cohort())
                    .expect("age supply source");
                let weight = id.select(&activity_sources);
                if complement {
                    LazySpotValuePerBlock::from_weighted_supply::<true>(
                        &name, version, supply, weight, mappings, spot_price,
                    )
                } else {
                    LazySpotValuePerBlock::from_weighted_supply::<false>(
                        &name, version, supply, weight, mappings, spot_price,
                    )
                }
            })
        };
        let supply = SupplyVecs {
            awake: import_supply("awake", false),
            dormant: import_supply("dormant", true),
        };
        Ok(Vecs {
            coindays_consumed,
            coindays_stored,
            activity,
            activity_sources,
            supply,
        })
    }
}
