use std::iter;

use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{DensitySeries, SupplyDensity};
use crate::distribution::{AgeCutoffs, DailyUrpds};

#[derive(Traversable)]
pub struct DensityMetrics<M: StorageMode = Rw> {
    pub all: DensitySeries<M>,
    #[traversable(flatten)]
    pub age: AgeCutoffs<DensitySeries<M>>,
}

impl DensityMetrics {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let import = |cohort: &str| {
            DensitySeries::forced_import(
                db,
                &format!("{name}_{cohort}_supply_density"),
                version,
                mappings,
            )
        };
        Ok(Self {
            all: import("all")?,
            age: AgeCutoffs::try_from_fn(import)?,
        })
    }

    pub fn push(&mut self, urpds: Option<&DailyUrpds>, spot: Cents) {
        let targets = iter::once(&mut self.all).chain(self.age.iter_mut());
        let sources = [
            urpds.map(|u| &u.all),
            urpds.map(|u| &u.age.under_4m),
            urpds.map(|u| &u.age.under_5m),
            urpds.map(|u| &u.age.under_6m),
        ];
        for (target, source) in targets.zip(sources) {
            let density = source
                .map(|u| SupplyDensity::from_entries(u.map.iter().map(|(&p, &s)| (p, s)), spot))
                .unwrap_or(SupplyDensity::NAN);
            target.push(&density);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        iter::once(&mut self.all)
            .chain(self.age.iter_mut())
            .flat_map(DensitySeries::stored_vecs_mut)
    }
}
