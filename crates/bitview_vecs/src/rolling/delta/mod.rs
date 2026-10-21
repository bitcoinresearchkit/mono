use brk_types::Version;

mod lazy_amount_from_height;
mod lazy_fiat_from_height;
mod lazy_from_height;
mod lazy_percent_from_height;
mod lazy_windows_from_height;

pub use lazy_amount_from_height::LazyDeltaAmountFromHeight;
pub use lazy_fiat_from_height::LazyDeltaFiatFromHeight;
pub use lazy_from_height::LazyDeltaFromHeight;
pub use lazy_percent_from_height::LazyDeltaPercentFromHeight;
pub use lazy_windows_from_height::{
    LazyRollingDeltas, LazyRollingDeltasAmountFromHeight, LazyRollingDeltasFiatFromHeight,
    LazyRollingDeltasFromHeight,
};

/// Window deltas and their percent changes start from the value before the window, subtracted exactly.
const VERSION: Version = Version::ONE;
