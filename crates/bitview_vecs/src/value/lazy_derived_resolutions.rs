use brk_types::{Bitcoin, Cents, Dollars, Sats, Version};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::{DerivedResolutions, Resolutions, SpotValueSource, Value};
use bitview_compute::ComputedVecValue;

pub trait ReadableResolutions<T>
where
    T: ComputedVecValue + JsonSchema,
{
    fn transformed<O, F>(&self, name: &str, version: Version) -> DerivedResolutions<O, T>
    where
        O: ComputedVecValue + JsonSchema,
        F: UnaryTransform<T, O>;
}

impl<T> ReadableResolutions<T> for Resolutions<T>
where
    T: ComputedVecValue + JsonSchema + 'static,
{
    fn transformed<O, F>(&self, name: &str, version: Version) -> DerivedResolutions<O, T>
    where
        O: ComputedVecValue + JsonSchema,
        F: UnaryTransform<T, O>,
    {
        DerivedResolutions::from_derived_computed::<F>(name, version, self)
    }
}

impl<T, S> ReadableResolutions<T> for DerivedResolutions<T, S>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S: ComputedVecValue + JsonSchema,
{
    fn transformed<O, F>(&self, name: &str, version: Version) -> DerivedResolutions<O, T>
    where
        O: ComputedVecValue + JsonSchema,
        F: UnaryTransform<T, O>,
    {
        DerivedResolutions::from_lazy::<F, S>(name, version, self)
    }
}

pub type LazyValueDerivedResolutions = Value<
    DerivedResolutions<Sats, Sats>,
    DerivedResolutions<Cents, Cents>,
    DerivedResolutions<Bitcoin, Sats>,
    DerivedResolutions<Dollars, Dollars>,
>;

impl LazyValueDerivedResolutions {
    pub(crate) fn from_spot_block_source<
        SatsTransform,
        BitcoinTransform,
        CentsTransform,
        DollarsTransform,
    >(
        name: &str,
        source: &impl SpotValueSource,
        version: Version,
    ) -> Self
    where
        SatsTransform: UnaryTransform<Sats, Sats>,
        BitcoinTransform: UnaryTransform<Sats, Bitcoin>,
        CentsTransform: UnaryTransform<Cents, Cents>,
        DollarsTransform: UnaryTransform<Dollars, Dollars>,
    {
        let sats = source
            .sats_resolutions()
            .transformed::<Sats, SatsTransform>(&format!("{name}_sats"), version);

        let btc = source
            .sats_resolutions()
            .transformed::<Bitcoin, BitcoinTransform>(name, version);

        let cents = source
            .cents_resolutions()
            .transformed::<Cents, CentsTransform>(&format!("{name}_cents"), version);

        let usd = source
            .usd_resolutions()
            .transformed::<Dollars, DollarsTransform>(&format!("{name}_usd"), version);

        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
