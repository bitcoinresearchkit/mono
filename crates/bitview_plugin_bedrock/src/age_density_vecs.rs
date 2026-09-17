use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_error::Result;
use brk_types::{PartsPerMillion32, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{AgeCutoffs, DensityBands, SupplyDensity, SupplyDensityVecs, WeightedPair};

#[derive(Traversable)]
pub struct AgeDensityVecs<M: StorageMode = Rw> {
    /// Density relative to each cohort's own weighted supply, using the same
    /// rounded price buckets and weighted-satoshi rounding as aggregate URPDs.
    #[traversable(flatten)]
    pub series: AgeCutoffs<WeightedPair<DensityBands<SupplyDensityVecs<M>>>>,
}

impl AgeDensityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        Ok(Self {
            series: AgeCutoffs::try_from_fn(|age| {
                WeightedPair::try_from_fn(|weight| {
                    DensityBands::try_from_fn(|band| {
                        SupplyDensityVecs::forced_import(
                            db,
                            &format!("bedrock_{}_{age}_supply_density{band}", weight.as_str()),
                            version,
                            mappings,
                        )
                    })
                })
            })?,
        })
    }

    pub fn prepare(
        &mut self,
        version: Version,
        recompute_from: usize,
        end: usize,
    ) -> Result<usize> {
        let mut start = recompute_from.min(end);
        for vec in self.stored_vecs_mut() {
            vec.any_validate_computed_version_or_reset(version)?;
            start = start.min(vec.len());
        }
        for vec in self.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        Ok(start)
    }

    pub fn push(
        &mut self,
        values: &AgeCutoffs<WeightedPair<DensityBands<SupplyDensity<PartsPerMillion32>>>>,
    ) {
        let targets = self
            .series
            .iter_mut()
            .flat_map(WeightedPair::iter_mut)
            .flat_map(DensityBands::iter_mut);
        let values = values
            .iter()
            .flat_map(WeightedPair::iter)
            .flat_map(DensityBands::iter);
        for (target, value) in targets.zip(values) {
            target.push(value);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.series
            .iter_mut()
            .flat_map(WeightedPair::iter_mut)
            .flat_map(DensityBands::iter_mut)
            .flat_map(SupplyDensityVecs::stored_vecs_mut)
    }
}
