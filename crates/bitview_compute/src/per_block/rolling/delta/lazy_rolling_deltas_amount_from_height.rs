use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    DeltaChange, DeltaRate, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec, VecValue,
};

use crate::{
    AmountType, CachedWindowStartVec, DerivedResolutions, FixedRatio, LazyPerBlock, Windows,
};

use super::{LazyDeltaAmountFromHeight, LazyDeltaFromHeight, LazyDeltaPercentFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue,
    C: AmountType,
    B: FixedRatio,
{
    /// Absolute change from the start of a trailing window through the
    /// represented block.
    pub absolute: Windows<LazyDeltaAmountFromHeight<S, C>>,
    /// Relative change from the start of a trailing window through the
    /// represented block, divided by the starting value. Returns zero when the
    /// starting value is zero.
    pub rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: AmountType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let source = source.read_only_boxed_clone();
        let (absolute, rate) = cached_starts
            .map_with_suffix(|suffix, cached_start| {
                let name = format!("{name}_{suffix}");
                let cached = cached_start.read_only_clone();
                let starts_version = cached.version();

                let sats_name = format!("{name}_sats");
                let height = LazyDeltaVec::<Height, S, C, DeltaChange>::new(
                    &sats_name,
                    version,
                    source.clone(),
                    starts_version,
                    {
                        let cached = cached.clone();
                        move || cached.snapshot()
                    },
                );
                let sats = LazyDeltaFromHeight::new(&sats_name, version, height, indexes);
                let btc =
                    LazyPerBlock {
                        height: LazyVec::transformed::<C::ToBitcoin>(
                            &name,
                            version,
                            sats.height.read_only_boxed_clone(),
                        ),
                        resolutions: Box::new(DerivedResolutions::from_derived_computed::<
                            C::ToBitcoin,
                        >(
                            &name, version, &sats.resolutions
                        )),
                    };
                let absolute = LazyDeltaAmountFromHeight { btc, sats };

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
