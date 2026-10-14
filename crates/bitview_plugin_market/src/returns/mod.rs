use bitview_plugin_blocks::LookbackVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_vecs::{
    LazyPercentPerBlock, LazyWindowVec, ops::RelativeChange, shapes::ByLookbackPeriod,
};
use brk_types::{Dollars, Height, Version};
use vecdb::BinaryTransform;

pub type Returns = ByLookbackPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>;

/// Spot-price returns over each trailing window: the represented block's price over the
/// window's starting price, minus one.
pub(crate) fn import(
    version: Version,
    mappings: &MappingsVecs,
    lookback: &LookbackVecs,
    prices: &PriceVecs,
) -> Returns {
    ByLookbackPeriod::from_fn(|name, days| {
        let metric_name = format!("price_return_{name}");
        let source = LazyWindowVec::<Height, Dollars, PartsPerMillionSigned64>::new(
            &format!("{metric_name}_ppm_source"),
            version,
            &prices.spot.usd.height,
            lookback.start_vec(days),
            false,
            |current, past, _| RelativeChange::<PartsPerMillionSigned64>::apply(current, past),
        );
        LazyPercentPerBlock::from_height_source(&metric_name, version, &source, mappings)
    })
}
