use bitview_traversable::Traversable;
use brk_types::StoredF64;

use bitview_compute::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Trailing 365-day transfer volume in satoshis divided by current all-chain
    /// supply in satoshis. Returns zero when supply is zero.
    pub native: LazyPerBlock<StoredF64>,
    /// Trailing 365-day transfer volume valued in cents divided by current
    /// all-chain market capitalization in cents. Returns zero when market
    /// capitalization is zero.
    pub fiat: LazyPerBlock<StoredF64>,
}
