use brk_error::{Error, Result};
use brk_types::Version;
use brk_types::{Cents, Height};
use vecdb::CachedBoxedVec;

use super::{ByLookbackPeriod, Vecs};
use crate::{
    indexes,
    internal::{CACHE_BUDGET, LazyWindowVec, Price},
    price,
};

impl Vecs {
    pub(crate) fn forced_import(
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &ByLookbackPeriod<CachedBoxedVec<Height, Height>>,
        prices: &price::Vecs,
    ) -> Result<Self> {
        let price_past =
            ByLookbackPeriod::try_from_period(cached_starts, |name, _days, window_starts| {
                let metric_name = format!("price_past_{name}");
                let source = LazyWindowVec::<Height, Cents, Cents>::new(
                    &format!("{metric_name}_cents_source"),
                    version,
                    prices.spot.cents.height.read_only_boxed_clone(),
                    window_starts.clone(),
                    false,
                    |_, past, _| past,
                );
                let source = CACHE_BUDGET.wrap(source);
                Ok::<_, Error>(Price::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        Ok(Self { price_past })
    }
}
