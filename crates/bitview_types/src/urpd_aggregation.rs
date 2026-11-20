use std::fmt;

use bitview_primitives::CentsCompact;
use brk_types::Cents;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Significant digits of the raw cost-basis price in cents. Exact below $100. Five digits made the models'
/// full recompute 3.3× slower because early blocks each occupied their own price slot.
pub const COST_BASIS_PRICE_DIGITS: u32 = 4;

/// Aggregation strategy for URPD buckets.
/// Options: raw (no aggregation), lin200/lin500/lin1000 (linear $200/$500/$1000),
/// log10/log50/log100/log200/log500/log1000/log2000 (logarithmic, that many buckets per decade).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
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

impl fmt::Display for UrpdAggregation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(
            match self {
                Self::Raw => "raw",
                Self::Lin200 => "lin200",
                Self::Lin500 => "lin500",
                Self::Lin1000 => "lin1000",
                Self::Log10 => "log10",
                Self::Log50 => "log50",
                Self::Log100 => "log100",
                Self::Log200 => "log200",
                Self::Log500 => "log500",
                Self::Log1000 => "log1000",
                Self::Log2000 => "log2000",
            },
            f,
        )
    }
}

impl UrpdAggregation {
    pub const ALL: [Self; 11] = [
        Self::Raw,
        Self::Lin200,
        Self::Lin500,
        Self::Lin1000,
        Self::Log10,
        Self::Log50,
        Self::Log100,
        Self::Log200,
        Self::Log500,
        Self::Log1000,
        Self::Log2000,
    ];

    /// The label of the interval containing an exact creation price.
    pub fn bucket_floor(&self, price: Cents) -> Cents {
        if price.is_nan() {
            return Cents::NAN;
        }
        Cents::new(self.interval(price).0)
    }

    /// (Label, first cent of the next interval). Building the store and labeling its
    /// projections both walk these intervals. Logarithmic rows with no whole cents vanish.
    pub fn interval(self, price: Cents) -> (u64, u64) {
        let cents = price.inner();
        let linear = |width: u64| {
            let floor = cents / width * width;
            (floor, floor.saturating_add(width))
        };
        let density = match self {
            Self::Raw => {
                let compact = CentsCompact::from(price);
                let label = compact
                    .round_to_significant(COST_BASIS_PRICE_DIGITS)
                    .inner();
                let digits = label.checked_ilog10().unwrap_or(0) + 1;
                let step = 10_u64.pow(digits.saturating_sub(COST_BASIS_PRICE_DIGITS));
                let next = u64::from(label) + (step / 2).max(1);
                // At the compact ceiling, rounding up would exceed the price range.
                let upper = if u64::from(label) + step >= u64::from(u32::MAX) {
                    u64::MAX
                } else {
                    next
                };
                return (u64::from(label), upper);
            }
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
        if cents == 0 {
            return (0, 1);
        }
        let edge = |row: u32| {
            if row.is_multiple_of(density) {
                10_u64.saturating_pow(row / density)
            } else {
                10_f64.powf(f64::from(row) / f64::from(density)).ceil() as u64
            }
        };
        let mut row = ((cents as f64).log10() * f64::from(density)) as u32;
        // Use the integer edges themselves to decide membership, including at exact decades.
        while edge(row) > cents {
            row -= 1;
        }
        while edge(row + 1) <= cents {
            row += 1;
        }
        (edge(row), edge(row + 1))
    }
}
