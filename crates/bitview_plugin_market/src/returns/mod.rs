use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_vecs::{LazyPercentPerBlock, ops::RelativeChange, shapes::ByLookbackPeriod};
use brk_types::{Dollars, Height, Version};
use vecdb::{LazyDeltaVec, ReadableCloneableVec};

pub type Returns = ByLookbackPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>;

/// Spot-price returns over each trailing window: the represented block's price over the
/// window's starting price, minus one.
pub(crate) fn import(version: Version, mappings: &MappingsVecs, prices: &PriceVecs) -> Returns {
    ByLookbackPeriod::from_fn(|name, days| {
        let metric_name = format!("price_return_{name}");
        let source = LazyDeltaVec::<
            Height,
            Dollars,
            PartsPerMillionSigned64,
            RelativeChange<PartsPerMillionSigned64>,
        >::new(
            &format!("{metric_name}_ppm_source"),
            version,
            prices.spot.usd.height.read_only_boxed_clone(),
            mappings.lookback.start_vec(days).read_only_boxed_clone(),
        );
        LazyPercentPerBlock::from_height_source(&metric_name, version, &source, mappings)
    })
}
