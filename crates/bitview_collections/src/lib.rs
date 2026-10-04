//! Typed shapes, field identities, and shape-preserving operations.
//! Concrete vectors and cache ownership belong to their consumers.

mod by_lookback_period;
mod by_percentile;
mod distribution_stats;
mod fixed_ratio_views;
mod ohlc;
mod per_resolution;
mod rarity_percentiles;
mod resolution_fields;
mod windows;
mod windows_from_1w;
mod windows_to_1m;

pub use by_lookback_period::{ByLookbackPeriod, LOOKBACK_PERIOD_DAYS, LOOKBACK_PERIOD_NAMES};
pub use by_percentile::ByPercentile;
pub use distribution_stats::DistributionStats;
pub use fixed_ratio_views::FixedRatioViews;
pub use ohlc::Ohlc;
pub use per_resolution::PerResolution;
pub use rarity_percentiles::RarityPercentiles;
pub use windows::Windows;
pub use windows_from_1w::WindowsFrom1w;
pub use windows_to_1m::WindowsTo1m;
