mod cost_basis_means;
mod lazy_per_block;
mod lazy_rolling_windows;
mod per_block;
mod price_with_mvrv;
mod price_with_ratio;
mod rolling_windows;

pub use cost_basis_means::{CostBasisMeans, CostBasisPerCoin, CostBasisPerDollar};
pub use lazy_per_block::LazyRatioPerBlock;
pub use lazy_rolling_windows::LazyRatioRollingWindows;
pub use per_block::RatioPerBlock;
pub use price_with_mvrv::PriceWithMvrv;
pub use price_with_ratio::PriceWithRatio;
pub use rolling_windows::RatioRollingWindows;

use bitview_primitives::{Multiple, Ratio};
use schemars::JsonSchema;
use vecdb::VecValue;

/// A ratio's public value: a plain [`Ratio`] or a [`Multiple`] (read as "times", 1 at parity).
pub trait RatioValue: VecValue + PartialOrd + JsonSchema + From<f32> {}

impl RatioValue for Ratio {}
impl RatioValue for Multiple {}
