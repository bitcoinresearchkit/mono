mod lazy;
mod lazy_cumulative_rolling;
mod lazy_per_block;
mod lazy_rolling_windows;
mod per_block;
mod rolling_windows;

pub use lazy::LazyFixedRatioVec;
pub use lazy_cumulative_rolling::LazyFixedRatioCumulativeRolling;
pub use lazy_per_block::LazyFixedRatioPerBlock;
pub use lazy_rolling_windows::LazyFixedRatioRollingWindows;
pub use per_block::FixedRatioPerBlock;
pub use rolling_windows::FixedRatioRollingWindows;
