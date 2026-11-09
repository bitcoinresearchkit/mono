use bitview_cohort::AgeAggregate;
use bitview_distribution::{WeightedCohortSides, WeightedCohortSources, WeightedCohortViews};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{AwakeVecs, CohortVecs, DormantVecs, Vecs};

const SIDES: WeightedCohortSides = WeightedCohortSides {
    weighted: "awake",
    complement: "dormant",
};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let sources = WeightedCohortSources::import(db, SIDES, version)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            WeightedCohortViews::new(db, id, SIDES, version, &sources, mappings, spot_price)
                .map(CohortVecs::new)
        })?;
        Ok(Vecs { cohorts, sources })
    }
}

impl CohortVecs {
    fn new(views: WeightedCohortViews) -> Self {
        Self {
            awake: AwakeVecs {
                supply: views.supply,
                supply_in_loss_share: views.supply_in_loss_share,
                capital: views.capital,
                realized_cap: views.realized_cap,
                cost_basis: views.cost_basis,
            },
            dormant: DormantVecs {
                supply: views.complement_supply,
            },
        }
    }
}
