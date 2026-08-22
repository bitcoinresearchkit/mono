use bitview_traversable::Traversable;
use brk_types::StoredF64;

use bitview_compute::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Trailing 365-day transfer volume in satoshis divided by all-chain supply
    /// in satoshis at the represented block. It estimates how many times one
    /// year's on-chain transfer volume turns over the current supply. Returns
    /// zero when supply is zero.
    pub native: LazyPerBlock<StoredF64>,
    /// Trailing 365-day transfer volume valued in cents divided by all-chain
    /// market capitalization in cents at the represented block. Returns zero
    /// when market capitalization is zero. It compares one year's transferred
    /// USD value with the current market value of the supply.
    pub fiat: LazyPerBlock<StoredF64>,
}
