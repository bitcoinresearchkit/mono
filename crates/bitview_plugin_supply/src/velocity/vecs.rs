use bitview_primitives::Ratio64;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Trailing 365-day transfer volume in satoshis divided by all-chain supply
    /// in satoshis at the represented block. It estimates how many times one
    /// year's on-chain transfer volume turns over the current supply. Returns
    /// zero when supply is zero.
    pub native: LazyPerBlock<Ratio64>,
    /// Trailing 365-day transfer volume valued in cents divided by all-chain
    /// market capitalization in cents at the represented block. Returns zero
    /// when market capitalization is zero. It compares one year's transferred
    /// USD value with the current market value of the supply.
    pub fiat: LazyPerBlock<Ratio64>,
}
