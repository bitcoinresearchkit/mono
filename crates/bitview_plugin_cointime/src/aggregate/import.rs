use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_distribution::{WeightedCohortSources, metrics::WeightedCohort};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazySpotValuePerBlock, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{CohortVecs, DormantVecs, Vecs};

impl Vecs {
    /// `group_caps`: each holder group's realized cap, its awake coins' capital share's whole.
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        group_caps: &AgeAggregate<ReadableBoxedVec<Height, Cents>>,
    ) -> Result<Self> {
        let sources = WeightedCohortSources::import(db, "awake", "dormant", version, mappings)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            CohortVecs::new(
                db,
                id,
                version,
                &sources,
                mappings,
                window_starts,
                spot_price,
                id.select(group_caps),
            )
        })?;
        Ok(Vecs { cohorts, sources })
    }
}

impl CohortVecs {
    #[allow(clippy::too_many_arguments)]
    fn new(
        db: &Database,
        id: AgeAggregateId,
        version: Version,
        sources: &WeightedCohortSources,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        group_cap: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        Ok(Self {
            awake: WeightedCohort::import(
                db,
                id,
                "awake",
                version,
                sources,
                mappings,
                window_starts,
                spot_price,
                group_cap,
            )?,
            dormant: DormantVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &id.metric_name("dormant_supply"),
                    version,
                    id.select(&sources.complement_supply),
                    mappings,
                    spot_price,
                ),
            },
        })
    }
}
