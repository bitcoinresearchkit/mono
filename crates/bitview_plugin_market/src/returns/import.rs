use bitview_collections::ByLookbackPeriod;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_transforms::RelativeChange;
use bitview_vecs::{LazyPercentPerBlock, LazyWindowVec, RollingAvgSd};
use brk_error::{Error, Result};
use brk_types::{Dollars, Height, Version};
use vecdb::{BinaryTransform, Database, ReadableCloneableVec};

use super::{Cagr, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &ByLookbackPeriod<&impl ReadableCloneableVec<Height, Height>>,
        prices: &PriceVecs,
    ) -> Result<Self> {
        let periods =
            ByLookbackPeriod::try_from_period(window_starts, |name, _days, window_starts| {
                let metric_name = format!("price_return_{name}");
                let source = LazyWindowVec::<Height, Dollars, PartsPerMillionSigned64>::new(
                    &format!("{metric_name}_ppm_source"),
                    version,
                    &prices.spot.usd.height,
                    *window_starts,
                    false,
                    |current, past, _| {
                        RelativeChange::<PartsPerMillionSigned64>::apply(current, past)
                    },
                );
                Ok::<_, Error>(LazyPercentPerBlock::from_height_source(
                    &metric_name,
                    version,
                    &source,
                    mappings,
                ))
            })?;

        let cagr = Cagr::new(version, &periods);

        let daily = RollingAvgSd::import(db, "price_return_24h", version + Version::ONE, mappings)?;

        Ok(Vecs {
            periods,
            cagr,
            daily,
        })
    }
}
