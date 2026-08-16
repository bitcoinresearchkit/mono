use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats, Version};
use schemars::JsonSchema;
use vecdb::UnaryTransform;

use crate::internal::{ComputedVecValue, DerivedResolutions, Resolutions, SpotValueSource};

pub(crate) trait ReadableResolutions<T>
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

#[derive(Clone, Traversable)]
pub struct LazyValueDerivedResolutions {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: DerivedResolutions<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: DerivedResolutions<Sats, Sats>,
    /// Reported in US dollars.
    pub usd: DerivedResolutions<Dollars, Dollars>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: DerivedResolutions<Cents, Cents>,
}

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
