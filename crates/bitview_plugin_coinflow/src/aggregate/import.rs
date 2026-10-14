use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_distribution::WeightedCohortSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{LazyFiatPerBlock, LazyPerBlock, LazySpotValuePerBlock, ops::FixedToPercent};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{CohortVecs, ImmobileVecs, MobileVecs, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let sources = WeightedCohortSources::import(db, "mobile", "immobile", version)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            CohortVecs::new(db, id, version, &sources, mappings, spot_price)
        })?;
        Ok(Vecs { cohorts, sources })
    }
}

impl CohortVecs {
    fn new(
        db: &Database,
        id: AgeAggregateId,
        version: Version,
        sources: &WeightedCohortSources,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric: &str| id.metric_name(metric);
        Ok(Self {
            mobile: MobileVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("mobile_supply"),
                    version,
                    id.select(&sources.weighted_supply),
                    mappings,
                    spot_price,
                ),
                supply_in_loss_share: LazyPerBlock::from_height_source::<FixedToPercent>(
                    &name("mobile_supply_in_loss_share"),
                    version,
                    id.select(&sources.weighted_supply_in_loss_share),
                    mappings,
                ),
                capital: LazyFiatPerBlock::from_cents_source(
                    &name("mobile_capital"),
                    version,
                    id.select(&sources.weighted_realized_cap),
                    mappings,
                ),
                realized_cap: LazyFiatPerBlock::from_cents_source(
                    &name("mobile_realized_cap"),
                    version,
                    id.select(&sources.weighted_realized_cap),
                    mappings,
                ),
                cost_basis: CostBasisVecs::import(
                    db,
                    &name("mobile"),
                    version,
                    mappings,
                    id.select(&sources.weighted_realized_price),
                    id.select(&sources.weighted_capitalized_price),
                )?,
            },
            immobile: ImmobileVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("immobile_supply"),
                    version,
                    id.select(&sources.complement_supply),
                    mappings,
                    spot_price,
                ),
            },
        })
    }
}
