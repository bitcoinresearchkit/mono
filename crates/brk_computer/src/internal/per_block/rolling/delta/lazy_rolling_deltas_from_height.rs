use brk_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{DeltaChange, DeltaRate, LazyDeltaVec, ReadOnlyClone, ReadableCloneableVec, VecValue};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, FixedRatio, NumericValue, Windows},
};

use super::{LazyDeltaFromHeight, LazyDeltaPercentFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue,
    C: NumericValue + JsonSchema,
    B: FixedRatio,
{
    pub absolute: Windows<LazyDeltaFromHeight<S, C, DeltaChange>>,
    pub rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let source = source.read_only_boxed_clone();
        let (absolute, rate) = cached_starts
            .map_with_suffix(|suffix, cached_start| {
                let name = format!("{name}_{suffix}");
                let cached = cached_start.read_only_clone();
                let starts_version = cached.version();

                let height = LazyDeltaVec::<Height, S, C, DeltaChange>::new(
                    &name,
                    version,
                    source.clone(),
                    starts_version,
                    {
                        let cached = cached.clone();
                        move || cached.snapshot()
                    },
                );
                let absolute = LazyDeltaFromHeight::new(&name, version, height, indexes);

                let ppm_name = format!("{name}_rate_{}", B::SUFFIX);
                let height = LazyDeltaVec::<Height, S, B, DeltaRate>::new(
                    &ppm_name,
                    version,
                    source.clone(),
                    starts_version,
                    move || cached.snapshot(),
                );
                let ppm = LazyDeltaFromHeight::new(&ppm_name, version, height, indexes);
                let rate = LazyDeltaPercentFromHeight::from_ppm(&name, version, ppm);

                (absolute, rate)
            })
            .unzip();

        Self { absolute, rate }
    }
}
