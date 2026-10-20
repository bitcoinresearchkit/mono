use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Trailing 365-day transfer volume in BTC divided by the all-chain supply
    /// at the represented block; undefined (null) when supply is zero.
    pub btc: LazyPerBlock<Ratio>,
    /// Trailing 365-day transfer volume valued in US dollars divided by the
    /// all-chain market capitalization at the represented block; undefined (null)
    /// when market capitalization is zero.
    pub usd: LazyPerBlock<Ratio>,
}
