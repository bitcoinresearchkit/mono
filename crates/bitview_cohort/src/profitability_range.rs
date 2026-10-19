#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CohortName;

/// The 24 band edges, ascending, as divisors of the spot price: a band's edge is the creation
/// price `spot × 100 / divisor` at which the profit or loss reaches its bound (`100 + P` for P%
/// profit, `100 - L` for L% loss). The break-even edge (divisor 100, spot itself) counts as profit.
pub const PROFITABILITY_EDGE_DIVISORS: [u64; 24] = [
    1100, 600, 400, 300, 200, 190, 180, 170, 160, 150, 140, 130, 120, 110, 100, 90, 80, 70, 60, 50,
    40, 30, 20, 10,
];

/// The break-even edge's divisor: creation prices at spot are in profit, like the holders'.
pub const BREAK_EVEN_DIVISOR: u64 = 100;

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

/// 25 profitability range buckets ordered from most profitable to most in loss.
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
    /// Uses UTXOs whose represented-block spot price equals creation price or
    /// is above it by no more than 10%.
    profit_0_to_10_percent: T,
    /// Uses UTXOs whose represented-block spot price is less than 10% below
    /// creation price.
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
