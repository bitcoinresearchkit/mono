#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Cents;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CohortName;

/// Number of profitability range boundaries (24 boundaries → 25 buckets).
const PROFITABILITY_BOUNDARY_COUNT: usize = 24;

/// Compute 24 boundary prices from spot price for profitability bucketing.
///
/// Boundaries are returned in ascending price order (most profitable first → least profitable last).
/// Bucket assignment: prices ascending in k-way merge means we start at the most-profitable bucket
/// (lowest cost basis = highest profit) and advance the cursor as price crosses each boundary.
///
/// For P% profit: boundary = spot × 100 / (100 + P)
/// For L% loss:   boundary = spot × 100 / (100 - L)
///
/// Returns boundaries in ascending order:
/// [spot/11, spot/6, spot/4, spot/3, spot/2, spot×100/190, spot×100/180, ..., spot×100/10]
pub fn compute_profitability_boundaries(spot: Cents) -> [Cents; PROFITABILITY_BOUNDARY_COUNT] {
    let s = spot.as_u128();
    // Divisors in ascending boundary order (ascending price):
    // profit_over_1000_percent: price < spot/11          → boundary at spot*100/1100 = spot/11
    // profit_500_to_1000_percent: spot/11 ≤ p < spot/6   → boundary at spot*100/600  = spot/6
    // profit_300_to_500_percent: spot/6 ≤ p < spot/4     → boundary at spot*100/400  = spot/4
    // profit_200_to_300_percent: spot/4 ≤ p < spot/3     → boundary at spot*100/300  = spot/3
    // profit_100_to_200_percent: spot/3 ≤ p < spot/2     → boundary at spot*100/200  = spot/2
    // profit_90_to_100_percent: spot/2 ≤ p < spot*100/190 → boundary at spot*100/190
    // profit_80_to_90_percent:                            → boundary at spot*100/180
    // profit_70_to_80_percent:                            → boundary at spot*100/170
    // profit_60_to_70_percent:                            → boundary at spot*100/160
    // profit_50_to_60_percent:                            → boundary at spot*100/150
    // profit_40_to_50_percent:                            → boundary at spot*100/140
    // profit_30_to_40_percent:                            → boundary at spot*100/130
    // profit_20_to_30_percent:                            → boundary at spot*100/120
    // profit_10_to_20_percent:                            → boundary at spot*100/110
    // profit_0_to_10_percent:                             → boundary at spot (= spot*100/100)
    // loss_0_to_10_percent: spot ≤ p < spot*100/90    → boundary at spot*100/90
    // loss_10_to_20_percent:                          → boundary at spot*100/80
    // loss_20_to_30_percent:                          → boundary at spot*100/70
    // loss_30_to_40_percent:                          → boundary at spot*100/60
    // loss_40_to_50_percent:                          → boundary at spot*100/50 = spot*2
    // loss_50_to_60_percent:                          → boundary at spot*100/40 = spot*5/2
    // loss_60_to_70_percent:                          → boundary at spot*100/30 = spot*10/3
    // loss_70_to_80_percent:                          → boundary at spot*100/20 = spot*5
    // loss_80_to_90_percent:                          → boundary at spot*100/10 = spot*10
    // loss_90_to_100_percent: spot*10 ≤ p              (no upper boundary)
    let divisors: [u128; PROFITABILITY_BOUNDARY_COUNT] = [
        1100, // >1000% profit upper bound (spot/11)
        600,  // 500-1000% profit upper bound (spot/6)
        400,  // 300-500% profit upper bound (spot/4)
        300,  // 200-300% profit upper bound (spot/3)
        200,  // 100-200% profit upper bound (spot/2)
        190,  // 90-100% profit upper bound
        180,  // 80-90% profit upper bound
        170,  // 70-80% profit upper bound
        160,  // 60-70% profit upper bound
        150,  // 50-60% profit upper bound
        140,  // 40-50% profit upper bound
        130,  // 30-40% profit upper bound
        120,  // 20-30% profit upper bound
        110,  // 10-20% profit upper bound
        100,  // 0-10% profit upper bound (= spot)
        90,   // 0-10% loss upper bound
        80,   // 10-20% loss upper bound
        70,   // 20-30% loss upper bound
        60,   // 30-40% loss upper bound
        50,   // 40-50% loss upper bound
        40,   // 50-60% loss upper bound
        30,   // 60-70% loss upper bound
        20,   // 70-80% loss upper bound
        10,   // 80-90% loss upper bound
    ];

    let mut boundaries = [Cents::ZERO; PROFITABILITY_BOUNDARY_COUNT];
    for (i, &d) in divisors.iter().enumerate() {
        boundaries[i] = Cents::from(s * 100 / d);
    }
    boundaries
}

/// Profitability range names (25 ranges, from most profitable to most in loss)
pub const PROFITABILITY_RANGE_NAMES: ProfitabilityRange<CohortName> = ProfitabilityRange {
    profit_over_1000_percent: CohortName::new(
        "profit_over_1000_percent",
        "+>1000%",
        "Over 1000% in Profit",
    ),
    profit_500_to_1000_percent: CohortName::new(
        "profit_500_to_1000_percent",
        "+500-1000%",
        "500-1000% in Profit",
    ),
    profit_300_to_500_percent: CohortName::new(
        "profit_300_to_500_percent",
        "+300-500%",
        "300-500% in Profit",
    ),
    profit_200_to_300_percent: CohortName::new(
        "profit_200_to_300_percent",
        "+200-300%",
        "200-300% in Profit",
    ),
    profit_100_to_200_percent: CohortName::new(
        "profit_100_to_200_percent",
        "+100-200%",
        "100-200% in Profit",
    ),
    profit_90_to_100_percent: CohortName::new(
        "profit_90_to_100_percent",
        "+90-100%",
        "90-100% in Profit",
    ),
    profit_80_to_90_percent: CohortName::new(
        "profit_80_to_90_percent",
        "+80-90%",
        "80-90% in Profit",
    ),
    profit_70_to_80_percent: CohortName::new(
        "profit_70_to_80_percent",
        "+70-80%",
        "70-80% in Profit",
    ),
    profit_60_to_70_percent: CohortName::new(
        "profit_60_to_70_percent",
        "+60-70%",
        "60-70% in Profit",
    ),
    profit_50_to_60_percent: CohortName::new(
        "profit_50_to_60_percent",
        "+50-60%",
        "50-60% in Profit",
    ),
    profit_40_to_50_percent: CohortName::new(
        "profit_40_to_50_percent",
        "+40-50%",
        "40-50% in Profit",
    ),
    profit_30_to_40_percent: CohortName::new(
        "profit_30_to_40_percent",
        "+30-40%",
        "30-40% in Profit",
    ),
    profit_20_to_30_percent: CohortName::new(
        "profit_20_to_30_percent",
        "+20-30%",
        "20-30% in Profit",
    ),
    profit_10_to_20_percent: CohortName::new(
        "profit_10_to_20_percent",
        "+10-20%",
        "10-20% in Profit",
    ),
    profit_0_to_10_percent: CohortName::new("profit_0_to_10_percent", "+0-10%", "0-10% in Profit"),
    loss_0_to_10_percent: CohortName::new("loss_0_to_10_percent", "-0-10%", "0-10% in Loss"),
    loss_10_to_20_percent: CohortName::new("loss_10_to_20_percent", "-10-20%", "10-20% in Loss"),
    loss_20_to_30_percent: CohortName::new("loss_20_to_30_percent", "-20-30%", "20-30% in Loss"),
    loss_30_to_40_percent: CohortName::new("loss_30_to_40_percent", "-30-40%", "30-40% in Loss"),
    loss_40_to_50_percent: CohortName::new("loss_40_to_50_percent", "-40-50%", "40-50% in Loss"),
    loss_50_to_60_percent: CohortName::new("loss_50_to_60_percent", "-50-60%", "50-60% in Loss"),
    loss_60_to_70_percent: CohortName::new("loss_60_to_70_percent", "-60-70%", "60-70% in Loss"),
    loss_70_to_80_percent: CohortName::new("loss_70_to_80_percent", "-70-80%", "70-80% in Loss"),
    loss_80_to_90_percent: CohortName::new("loss_80_to_90_percent", "-80-90%", "80-90% in Loss"),
    loss_90_to_100_percent: CohortName::new(
        "loss_90_to_100_percent",
        "-90-100%",
        "90-100% in Loss",
    ),
};

impl ProfitabilityRange<CohortName> {
    pub const fn names() -> &'static Self {
        &PROFITABILITY_RANGE_NAMES
    }
}

/// 25 profitability range buckets ordered from most profitable to most in loss.
///
/// During the k-way merge (ascending price order), the cursor starts at bucket 0
/// (profit_over_1000_percent, lowest cost basis) and advances as price crosses each boundary.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ProfitabilityRange<T> {
    /// Uses UTXOs whose represented-block spot price is more than 1,000% above
    /// creation price.
    profit_over_1000_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 500% and no
    /// more than 1,000% above creation price.
    profit_500_to_1000_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 300% and no
    /// more than 500% above creation price.
    profit_300_to_500_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 200% and no
    /// more than 300% above creation price.
    profit_200_to_300_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 100% and no
    /// more than 200% above creation price.
    profit_100_to_200_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 90% and no
    /// more than 100% above creation price.
    profit_90_to_100_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 80% and no
    /// more than 90% above creation price.
    profit_80_to_90_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 70% and no
    /// more than 80% above creation price.
    profit_70_to_80_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 60% and no
    /// more than 70% above creation price.
    profit_60_to_70_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 50% and no
    /// more than 60% above creation price.
    profit_50_to_60_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 40% and no
    /// more than 50% above creation price.
    profit_40_to_50_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 30% and no
    /// more than 40% above creation price.
    profit_30_to_40_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 20% and no
    /// more than 30% above creation price.
    profit_20_to_30_percent: T,
    /// Uses UTXOs whose represented-block spot price is more than 10% and no
    /// more than 20% above creation price.
    profit_10_to_20_percent: T,
    /// Uses UTXOs whose represented-block spot price is above creation price by
    /// no more than 10%.
    profit_0_to_10_percent: T,
    /// Uses UTXOs whose represented-block spot price equals creation price or
    /// is less than 10% below it.
    loss_0_to_10_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 10% and less
    /// than 20% below creation price.
    loss_10_to_20_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 20% and less
    /// than 30% below creation price.
    loss_20_to_30_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 30% and less
    /// than 40% below creation price.
    loss_30_to_40_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 40% and less
    /// than 50% below creation price.
    loss_40_to_50_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 50% and less
    /// than 60% below creation price.
    loss_50_to_60_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 60% and less
    /// than 70% below creation price.
    loss_60_to_70_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 70% and less
    /// than 80% below creation price.
    loss_70_to_80_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 80% and less
    /// than 90% below creation price.
    loss_80_to_90_percent: T,
    /// Uses UTXOs whose represented-block spot price is at least 90% below
    /// creation price.
    loss_90_to_100_percent: T,
}

define_cohort_id!(
    ProfitabilityRangeId for ProfitabilityRange {
        ProfitOver1000Percent => profit_over_1000_percent,
        Profit500To1000Percent => profit_500_to_1000_percent,
        Profit300To500Percent => profit_300_to_500_percent,
        Profit200To300Percent => profit_200_to_300_percent,
        Profit100To200Percent => profit_100_to_200_percent,
        Profit90To100Percent => profit_90_to_100_percent,
        Profit80To90Percent => profit_80_to_90_percent,
        Profit70To80Percent => profit_70_to_80_percent,
        Profit60To70Percent => profit_60_to_70_percent,
        Profit50To60Percent => profit_50_to_60_percent,
        Profit40To50Percent => profit_40_to_50_percent,
        Profit30To40Percent => profit_30_to_40_percent,
        Profit20To30Percent => profit_20_to_30_percent,
        Profit10To20Percent => profit_10_to_20_percent,
        Profit0To10Percent => profit_0_to_10_percent,
        Loss0To10Percent => loss_0_to_10_percent,
        Loss10To20Percent => loss_10_to_20_percent,
        Loss20To30Percent => loss_20_to_30_percent,
        Loss30To40Percent => loss_30_to_40_percent,
        Loss40To50Percent => loss_40_to_50_percent,
        Loss50To60Percent => loss_50_to_60_percent,
        Loss60To70Percent => loss_60_to_70_percent,
        Loss70To80Percent => loss_70_to_80_percent,
        Loss80To90Percent => loss_80_to_90_percent,
        Loss90To100Percent => loss_90_to_100_percent,
    }
);
