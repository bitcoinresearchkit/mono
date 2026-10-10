use bitview_collections::ByLookbackPeriod;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::{LazyWindowVec, Price};
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Version};
use vecdb::ReadableCloneableVec;

use super::Vecs;

impl Vecs {
    pub(crate) fn new(
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &ByLookbackPeriod<&impl ReadableCloneableVec<Height, Height>>,
        prices: &PriceVecs,
    ) -> Result<Self> {
        let price_past =
            ByLookbackPeriod::try_from_period(window_starts, |name, _days, window_starts| {
                let metric_name = format!("price_{name}_ago");
                let source = LazyWindowVec::<Height, Cents, Cents>::new(
                    &format!("{metric_name}_cents_source"),
                    version,
                    &prices.spot.cents.height,
                    *window_starts,
                    false,
                    |_, past, _| past,
                );
                Ok::<_, Error>(Price::from_height_source(
                    &metric_name,
                    version,
                    &source,
                    mappings,
                ))
            })?;

        Ok(Vecs { price_past })
    }
}
