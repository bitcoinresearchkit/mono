use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::FixedToPercent;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyPerBlock, LazySpotValuePerBlock, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, PcoVecValue, ReadableBoxedVec};

use super::{AwakeVecs, CohortVecs, DormantVecs, Sources, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let sources = Sources::import(db, version)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            CohortVecs::new(db, id, version, &sources, mappings, spot_price)
        })?;
        Ok(Vecs { cohorts, sources })
    }
}

impl Sources {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            awake_supply: import_aggregate(db, "awake_supply_sats", version)?,
            dormant_supply: import_aggregate(db, "dormant_supply_sats", version)?,
            awake_realized_cap: import_aggregate(db, "awake_realized_cap_cents", version)?,
            awake_realized_price: import_aggregate(db, "awake_realized_price_cents", version)?,
            awake_capitalized_price: import_aggregate(
                db,
                "awake_capitalized_price_cents",
                version,
            )?,
            awake_supply_in_loss_share: import_aggregate(
                db,
                "awake_supply_in_loss_share_bounded",
                version,
            )?,
        })
    }
}

fn import_aggregate<T: PcoVecValue>(
    db: &Database,
    metric: &str,
    version: Version,
) -> Result<AgeAggregate<CachedSeries<Height, T>>> {
    AgeAggregate::try_from_fn(|id| import_cached(db, &id.metric_name(metric), version))
}

impl CohortVecs {
    fn new(
        db: &Database,
        id: AgeAggregateId,
        version: Version,
        sources: &Sources,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric: &str| id.metric_name(metric);
        Ok(Self {
            awake: AwakeVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("awake_supply"),
                    version,
                    id.select(&sources.awake_supply),
                    mappings,
                    spot_price,
                ),
                supply_in_loss_share: LazyPerBlock::from_height_source::<FixedToPercent>(
                    &name("awake_supply_in_loss_share"),
                    version,
                    id.select(&sources.awake_supply_in_loss_share),
                    mappings,
                ),
                capital: LazyFiatPerBlock::from_cents_source(
                    &name("awake_capital"),
                    version,
                    id.select(&sources.awake_realized_cap),
                    mappings,
                ),
                realized_cap: LazyFiatPerBlock::from_cents_source(
                    &name("awake_realized_cap"),
                    version,
                    id.select(&sources.awake_realized_cap),
                    mappings,
                ),
                cost_basis: CostBasisVecs::import(
                    db,
                    &name("awake"),
                    version,
                    mappings,
                    id.select(&sources.awake_realized_price),
                    id.select(&sources.awake_capitalized_price),
                )?,
            },
            dormant: DormantVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("dormant_supply"),
                    version,
                    id.select(&sources.dormant_supply),
                    mappings,
                    spot_price,
                ),
            },
        })
    }
}
