use bitview_compute::{
    CentsUnsignedToDollars, Identity, LazyPerBlock, LazyWindowVec, SatsToBitcoin, SatsToCents,
};
use bitview_plugin_mappings::Vecs as MappingVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{BinaryTransform, CachedBoxedVec};

use crate::DCA_AMOUNT;

#[derive(Clone, Traversable)]
pub struct LumpSumStack {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: LazyPerBlock<Sats>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyPerBlock<Cents>,
}

impl LumpSumStack {
    pub fn from_window(
        name: &str,
        days: u32,
        version: Version,
        mappings: &MappingVecs,
        window_starts: &CachedBoxedVec<Height, Height>,
        prices: &PriceVecs,
    ) -> Result<Self> {
        let total_invested = DCA_AMOUNT * days as usize;

        let sats_source = LazyWindowVec::<Height, Cents, Sats>::new(
            &format!("{name}_sats_source"),
            version,
            prices.spot.cents.height.read_only_boxed_clone(),
            window_starts.clone(),
            false,
            move |_, past, _| Self::sats_at_price(total_invested, past),
        );
        let sats = LazyPerBlock::from_height_source::<Identity<Sats>>(
            &format!("{name}_sats"),
            version,
            sats_source,
            mappings,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);

        let cents_source = LazyWindowVec::<Height, Cents, Cents>::new(
            &format!("{name}_cents_source"),
            version,
            prices.spot.cents.height.read_only_boxed_clone(),
            window_starts.clone(),
            false,
            move |current, past, _| {
                SatsToCents::apply(Self::sats_at_price(total_invested, past), current)
            },
        );
        let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
            &format!("{name}_cents"),
            version,
            cents_source,
            mappings,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            &format!("{name}_usd"),
            version,
            &cents,
        );

        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }

    #[inline(always)]
    fn sats_at_price(total_invested: Dollars, price: Cents) -> Sats {
        Sats::from_dollars_at_price(total_invested, Dollars::from(price))
    }
}
