use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::Cents;

/// Aggregation strategy for URPD buckets.
/// Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
/// log10/log50/log100/log200/log500/log1000/log2000 (logarithmic with 10/50/100/200/500/1000/2000 buckets per decade).
#[derive(
    Debug, Display, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum UrpdAggregation {
    #[default]
    Raw,
    Lin200,
    Lin500,
    Lin1000,
    Log10,
    Log50,
    Log100,
    Log200,
    Log500,
    Log1000,
    Log2000,
}

impl UrpdAggregation {
    /// Returns the linear bucket size in cents, if this is a linear bucket type.
    fn linear_size_cents(&self) -> Option<u64> {
        match self {
            Self::Lin200 => Some(20_000),
            Self::Lin500 => Some(50_000),
            Self::Lin1000 => Some(100_000),
            _ => None,
        }
    }

    /// Returns the number of buckets per decade, if this is a log bucket type.
    fn log_buckets_per_decade(&self) -> Option<u32> {
        match self {
            Self::Log10 => Some(10),
            Self::Log50 => Some(50),
            Self::Log100 => Some(100),
            Self::Log200 => Some(200),
            Self::Log500 => Some(500),
            Self::Log1000 => Some(1000),
            Self::Log2000 => Some(2000),
            _ => None,
        }
    }

    /// Compute the bucket floor for a price in cents.
    /// `Raw` is the identity (no bucketing).
    pub fn bucket_floor(&self, price_cents: Cents) -> Cents {
        match self {
            Self::Raw => price_cents,
            Self::Lin200 | Self::Lin500 | Self::Lin1000 => {
                let size = self.linear_size_cents().unwrap();
                (price_cents / size) * size
            }
            Self::Log10
            | Self::Log50
            | Self::Log100
            | Self::Log200
            | Self::Log500
            | Self::Log1000
            | Self::Log2000 => {
                if price_cents == Cents::ZERO {
                    return Cents::ZERO;
                }
                let n = self.log_buckets_per_decade().unwrap();
                let log_price = f64::from(price_cents).log10();
                let bucket_idx = (n as f64 * log_price).floor() as i32;
                let floor = 10_f64.powf(bucket_idx as f64 / n as f64);
                Cents::from(floor.round() as u64)
            }
        }
    }
}
