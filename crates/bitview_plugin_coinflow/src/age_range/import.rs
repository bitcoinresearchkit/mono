use bitview_cohort::{AgeRange, CohortContext};
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::BoundedToRatio;
use bitview_vecs::{LazyPerBlock, LazySpotValuePerBlock, PerBlock, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{RangeVecs, SideVecs, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        age: &AgeVecs,
    ) -> Result<Self> {
        let ranges = AgeRange::try_from_fn(|id| -> Result<_> {
            let name = CohortContext::Utxo.full_name(id.cohort());
            let name = |metric: &str| format!("{name}_{metric}");
            let mobility_source = import_cached(db, &name("mobility_bounded_source"), version)?;
            let supply = &id.select(&age.ranges).supply.total.stored;
            Ok(RangeVecs {
                spending_rate: PerBlock::import(db, &name("spending_rate"), version, mappings)?,
                spending_exposure: PerBlock::import(
                    db,
                    &name("spending_exposure"),
                    version,
                    mappings,
                )?,
                mobility: LazyPerBlock::from_height_source::<BoundedToRatio>(
                    &name("mobility"),
                    version,
                    &mobility_source,
                    mappings,
                ),
                mobile: SideVecs {
                    supply: LazySpotValuePerBlock::from_weighted_supply::<false>(
                        &name("mobile_supply"),
                        version,
                        supply,
                        &mobility_source,
                        mappings,
                        spot_price,
                    ),
                },
                immobile: SideVecs {
                    supply: LazySpotValuePerBlock::from_weighted_supply::<true>(
                        &name("immobile_supply"),
                        version,
                        supply,
                        &mobility_source,
                        mappings,
                        spot_price,
                    ),
                },
                mobility_source,
            })
        })?;
        Ok(Vecs { ranges })
    }
}
