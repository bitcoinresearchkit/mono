use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::FixedToPercent;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock,
    LazySpotValuePerBlock, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, PcoVecValue, ReadableBoxedVec};

use super::{CohortVecs, ImmobileVecs, MobileVecs, Sources, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let sources = Sources::import(db, version)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            CohortVecs::new(db, id, version, &sources, mappings, spot_price)
        })?;
        Ok(Vecs { cohorts, sources })
    }
}

impl Sources {
    fn import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            mobile_supply: import_aggregate(db, "mobile_supply_sats", version)?,
            immobile_supply: import_aggregate(db, "immobile_supply_sats", version)?,
            mobile_realized_cap: import_aggregate(db, "mobile_realized_cap_cents", version)?,
            mobile_realized_price: import_aggregate(db, "mobile_realized_price_cents", version)?,
            mobile_capitalized_price: import_aggregate(
                db,
                "mobile_capitalized_price_cents",
                version,
            )?,
            mobile_supply_in_loss_share: import_aggregate(
                db,
                "mobile_supply_in_loss_share_bounded",
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
            mobile: MobileVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("mobile_supply"),
                    version,
                    id.select(&sources.mobile_supply),
                    mappings,
                    spot_price,
                ),
                supply_in_loss_share: LazyPerBlock::from_height_source::<FixedToPercent>(
                    &name("mobile_supply_in_loss_share"),
                    version,
                    id.select(&sources.mobile_supply_in_loss_share),
                    mappings,
                ),
                realized_cap: LazyFiatPerBlock::from_cents_source(
                    &name("mobile_realized_cap"),
                    version,
                    id.select(&sources.mobile_realized_cap),
                    mappings,
                ),
                realized_price: LazyPriceWithRatioPerBlock::from_height_source(
                    &name("mobile_realized_price"),
                    version,
                    id.select(&sources.mobile_realized_price),
                    mappings,
                    spot_price,
                ),
                capitalized_price: LazyPriceWithRatioPerBlock::from_height_source(
                    &name("mobile_capitalized_price"),
                    version,
                    id.select(&sources.mobile_capitalized_price),
                    mappings,
                    spot_price,
                ),
                cost_basis: CostBasisVecs::import(db, &name("mobile"), version, mappings)?,
            },
            immobile: ImmobileVecs {
                supply: LazySpotValuePerBlock::from_sats_source(
                    &name("immobile_supply"),
                    version,
                    id.select(&sources.immobile_supply),
                    mappings,
                    spot_price,
                ),
            },
        })
    }
}
