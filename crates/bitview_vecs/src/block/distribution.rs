use bitview_collections::DistributionStats;
use bitview_compute::{ComputedVecValue, NumericValue};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, VSize, get_percentile, get_weighted_percentiles};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Budgeted, Database, EagerVec, PcoVec, Rw, StorageMode, Version, WritableVec};

use crate::{IndexSources, PerBlock};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct PerBlockDistribution<T: ComputedVecValue + PartialOrd + JsonSchema, M: StorageMode = Rw>(
    pub DistributionStats<PerBlock<T, M>>,
);

impl<T: NumericValue + JsonSchema> PerBlockDistribution<T> {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|suffix| {
            PerBlock::import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }

    // Preserve the validation, truncation, and write order of the stored outputs.
    #[inline]
    pub(crate) fn height_vecs_mut(&mut self) -> [&mut EagerVec<PcoVec<Height, T, Budgeted>>; 7] {
        [
            &mut self.0.min.height,
            &mut self.0.max.height,
            &mut self.0.median.height,
            &mut self.0.pct10.height,
            &mut self.0.pct25.height,
            &mut self.0.pct75.height,
            &mut self.0.pct90.height,
        ]
    }

    // Keep this hot output path in the caller, as before extracting the helper.
    #[inline(always)]
    pub(crate) fn push_sorted(&mut self, values: &[T]) {
        if let (Some(&min), Some(&max)) = (values.first(), values.last()) {
            self.max.height.push(max);
            self.pct90.height.push(get_percentile(values, 0.90));
            self.pct75.height.push(get_percentile(values, 0.75));
            self.median.height.push(get_percentile(values, 0.50));
            self.pct25.height.push(get_percentile(values, 0.25));
            self.pct10.height.push(get_percentile(values, 0.10));
            self.min.height.push(min);
        } else {
            for vec in self.height_vecs_mut() {
                vec.push(T::from(0_usize));
            }
        }
    }

    #[inline]
    pub(crate) fn push_weighted_sorted(&mut self, values: &[(T, VSize)]) {
        if let (Some(&(min, _)), Some(&(max, _))) = (values.first(), values.last()) {
            self.max.height.push(max);
            let [pct10, pct25, median, pct75, pct90] =
                get_weighted_percentiles(values, [0.10, 0.25, 0.50, 0.75, 0.90]);
            self.pct90.height.push(pct90);
            self.pct75.height.push(pct75);
            self.median.height.push(median);
            self.pct25.height.push(pct25);
            self.pct10.height.push(pct10);
            self.min.height.push(min);
        } else {
            for vec in self.height_vecs_mut() {
                vec.push(T::from(0_usize));
            }
        }
    }
}
