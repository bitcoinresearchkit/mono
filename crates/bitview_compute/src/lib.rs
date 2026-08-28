#![allow(clippy::type_complexity)]

mod age_band;
pub mod algo;
mod block_walker;
mod by_dca_cagr;
mod by_dca_period;
mod by_lookback_period;
mod cache_budget;
mod containers;
mod daily_metric;
pub mod db_utils;
mod index_sources;
mod lazy_indexes;
mod per_block;
mod per_tx;
mod percentile_prices;
mod traits;
mod transform;
mod value;
mod weighted;
mod with_addr_types;

pub use age_band::{AgeBand, MINIMUM_DURATION_DAYS};
pub use block_walker::*;
pub use by_dca_cagr::*;
pub use by_dca_period::*;
pub use by_lookback_period::*;
pub use cache_budget::*;
pub use containers::*;
pub use daily_metric::*;
pub use index_sources::*;
pub use lazy_indexes::*;
pub use per_block::*;
pub use per_tx::*;
pub use percentile_prices::*;
pub use traits::*;
pub use transform::*;
pub use value::*;
pub use weighted::{WeightedCohortContribution, WeightedCohortState, WeightedRatio};
pub use with_addr_types::*;

pub const TARGET_BLOCKS_PER_DAY_F64: f64 = 144.0;
pub const TARGET_BLOCKS_PER_DAY_F32: f32 = 144.0;
pub const TARGET_BLOCKS_PER_DAY: u64 = 144;
pub const TARGET_BLOCKS_PER_WEEK: u64 = 7 * TARGET_BLOCKS_PER_DAY;
pub const TARGET_BLOCKS_PER_MONTH: u64 = 30 * TARGET_BLOCKS_PER_DAY;
pub const TARGET_BLOCKS_PER_YEAR: u64 = 365 * TARGET_BLOCKS_PER_DAY;
