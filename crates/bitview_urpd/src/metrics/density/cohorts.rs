use std::iter;

use bitview_traversable::Traversable;
use bitview_vecs::IndexSources;
use brk_error::Result;
use brk_types::{PartsPerMillion32, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{DensitySeries, SupplyDensity};
use crate::distribution::AgeCutoffs;

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
        mappings: &IndexSources,
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

    pub fn push(&mut self, densities: Option<&[SupplyDensity<PartsPerMillion32>; 4]>) {
        let targets = iter::once(&mut self.all).chain(self.age.iter_mut());
        for (target, density) in targets.zip(densities.unwrap_or(&[SupplyDensity::NAN; 4])) {
            target.push(density);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        iter::once(&mut self.all)
            .chain(self.age.iter_mut())
            .flat_map(DensitySeries::stored_vecs_mut)
    }
}
