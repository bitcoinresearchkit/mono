use brk_types::Cents;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

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
    /// Compute the bucket floor for a price in cents.
    /// `Raw` is the identity (no bucketing).
    pub fn bucket_floor(&self, price_cents: Cents) -> Cents {
        let linear = |size: u64| price_cents / size * size;
        let buckets_per_decade: u32 = match self {
            Self::Raw => return price_cents,
            Self::Lin200 => return linear(20_000),
            Self::Lin500 => return linear(50_000),
            Self::Lin1000 => return linear(100_000),
            Self::Log10 => 10,
            Self::Log50 => 50,
            Self::Log100 => 100,
            Self::Log200 => 200,
            Self::Log500 => 500,
            Self::Log1000 => 1000,
            Self::Log2000 => 2000,
        };
        if price_cents == Cents::ZERO {
            return Cents::ZERO;
        }
        let n = f64::from(buckets_per_decade);
        let bucket_idx = (n * f64::from(price_cents).log10()).floor();
        Cents::from(10_f64.powf(bucket_idx / n))
    }
}
