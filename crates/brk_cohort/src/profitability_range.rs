use brk_traversable::Traversable;
use brk_types::Cents;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::{CohortName, PROFIT_COUNT};

/// Number of profitability range boundaries (24 boundaries → 25 buckets).
pub const PROFITABILITY_BOUNDARY_COUNT: usize = 24;

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
    // over_1000pct_in_profit: price < spot/11          → boundary at spot*100/1100 = spot/11
    // 500pct_to_1000pct_in_profit: spot/11 ≤ p < spot/6   → boundary at spot*100/600  = spot/6
    // 300pct_to_500pct_in_profit: spot/6 ≤ p < spot/4     → boundary at spot*100/400  = spot/4
    // 200pct_to_300pct_in_profit: spot/4 ≤ p < spot/3     → boundary at spot*100/300  = spot/3
    // 100pct_to_200pct_in_profit: spot/3 ≤ p < spot/2     → boundary at spot*100/200  = spot/2
    // 90pct_to_100pct_in_profit: spot/2 ≤ p < spot*100/190 → boundary at spot*100/190
    // 80pct_to_90pct_in_profit:                            → boundary at spot*100/180
    // 70pct_to_80pct_in_profit:                            → boundary at spot*100/170
    // 60pct_to_70pct_in_profit:                            → boundary at spot*100/160
    // 50pct_to_60pct_in_profit:                            → boundary at spot*100/150
    // 40pct_to_50pct_in_profit:                            → boundary at spot*100/140
    // 30pct_to_40pct_in_profit:                            → boundary at spot*100/130
    // 20pct_to_30pct_in_profit:                            → boundary at spot*100/120
    // 10pct_to_20pct_in_profit:                            → boundary at spot*100/110
    // 0pct_to_10pct_in_profit:                             → boundary at spot (= spot*100/100)
    // 0pct_to_10pct_in_loss: spot ≤ p < spot*100/90    → boundary at spot*100/90
    // 10pct_to_20pct_in_loss:                          → boundary at spot*100/80
    // 20pct_to_30pct_in_loss:                          → boundary at spot*100/70
    // 30pct_to_40pct_in_loss:                          → boundary at spot*100/60
    // 40pct_to_50pct_in_loss:                          → boundary at spot*100/50 = spot*2
    // 50pct_to_60pct_in_loss:                          → boundary at spot*100/40 = spot*5/2
    // 60pct_to_70pct_in_loss:                          → boundary at spot*100/30 = spot*10/3
    // 70pct_to_80pct_in_loss:                          → boundary at spot*100/20 = spot*5
    // 80pct_to_90pct_in_loss:                          → boundary at spot*100/10 = spot*10
    // 90pct_to_100pct_in_loss: spot*10 ≤ p              (no upper boundary)
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
    over_1000pct_in_profit: CohortName::new(
        "utxos_over_1000pct_in_profit",
        "+>1000%",
        "Over 1000% in Profit",
    ),
    _500pct_to_1000pct_in_profit: CohortName::new(
        "utxos_500pct_to_1000pct_in_profit",
        "+500-1000%",
        "500-1000% in Profit",
    ),
    _300pct_to_500pct_in_profit: CohortName::new(
        "utxos_300pct_to_500pct_in_profit",
        "+300-500%",
        "300-500% in Profit",
    ),
    _200pct_to_300pct_in_profit: CohortName::new(
        "utxos_200pct_to_300pct_in_profit",
        "+200-300%",
        "200-300% in Profit",
    ),
    _100pct_to_200pct_in_profit: CohortName::new(
        "utxos_100pct_to_200pct_in_profit",
        "+100-200%",
        "100-200% in Profit",
    ),
    _90pct_to_100pct_in_profit: CohortName::new(
        "utxos_90pct_to_100pct_in_profit",
        "+90-100%",
        "90-100% in Profit",
    ),
    _80pct_to_90pct_in_profit: CohortName::new(
        "utxos_80pct_to_90pct_in_profit",
        "+80-90%",
        "80-90% in Profit",
    ),
    _70pct_to_80pct_in_profit: CohortName::new(
        "utxos_70pct_to_80pct_in_profit",
        "+70-80%",
        "70-80% in Profit",
    ),
    _60pct_to_70pct_in_profit: CohortName::new(
        "utxos_60pct_to_70pct_in_profit",
        "+60-70%",
        "60-70% in Profit",
    ),
    _50pct_to_60pct_in_profit: CohortName::new(
        "utxos_50pct_to_60pct_in_profit",
        "+50-60%",
        "50-60% in Profit",
    ),
    _40pct_to_50pct_in_profit: CohortName::new(
        "utxos_40pct_to_50pct_in_profit",
        "+40-50%",
        "40-50% in Profit",
    ),
    _30pct_to_40pct_in_profit: CohortName::new(
        "utxos_30pct_to_40pct_in_profit",
        "+30-40%",
        "30-40% in Profit",
    ),
    _20pct_to_30pct_in_profit: CohortName::new(
        "utxos_20pct_to_30pct_in_profit",
        "+20-30%",
        "20-30% in Profit",
    ),
    _10pct_to_20pct_in_profit: CohortName::new(
        "utxos_10pct_to_20pct_in_profit",
        "+10-20%",
        "10-20% in Profit",
    ),
    _0pct_to_10pct_in_profit: CohortName::new(
        "utxos_0pct_to_10pct_in_profit",
        "+0-10%",
        "0-10% in Profit",
    ),
    _0pct_to_10pct_in_loss: CohortName::new(
        "utxos_0pct_to_10pct_in_loss",
        "-0-10%",
        "0-10% in Loss",
    ),
    _10pct_to_20pct_in_loss: CohortName::new(
        "utxos_10pct_to_20pct_in_loss",
        "-10-20%",
        "10-20% in Loss",
    ),
    _20pct_to_30pct_in_loss: CohortName::new(
        "utxos_20pct_to_30pct_in_loss",
        "-20-30%",
        "20-30% in Loss",
    ),
    _30pct_to_40pct_in_loss: CohortName::new(
        "utxos_30pct_to_40pct_in_loss",
        "-30-40%",
        "30-40% in Loss",
    ),
    _40pct_to_50pct_in_loss: CohortName::new(
        "utxos_40pct_to_50pct_in_loss",
        "-40-50%",
        "40-50% in Loss",
    ),
    _50pct_to_60pct_in_loss: CohortName::new(
        "utxos_50pct_to_60pct_in_loss",
        "-50-60%",
        "50-60% in Loss",
    ),
    _60pct_to_70pct_in_loss: CohortName::new(
        "utxos_60pct_to_70pct_in_loss",
        "-60-70%",
        "60-70% in Loss",
    ),
    _70pct_to_80pct_in_loss: CohortName::new(
        "utxos_70pct_to_80pct_in_loss",
        "-70-80%",
        "70-80% in Loss",
    ),
    _80pct_to_90pct_in_loss: CohortName::new(
        "utxos_80pct_to_90pct_in_loss",
        "-80-90%",
        "80-90% in Loss",
    ),
    _90pct_to_100pct_in_loss: CohortName::new(
        "utxos_90pct_to_100pct_in_loss",
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
/// (over_1000pct_in_profit, lowest cost basis) and advances as price crosses each boundary.
#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct ProfitabilityRange<T> {
    pub over_1000pct_in_profit: T,
    pub _500pct_to_1000pct_in_profit: T,
    pub _300pct_to_500pct_in_profit: T,
    pub _200pct_to_300pct_in_profit: T,
    pub _100pct_to_200pct_in_profit: T,
    pub _90pct_to_100pct_in_profit: T,
    pub _80pct_to_90pct_in_profit: T,
    pub _70pct_to_80pct_in_profit: T,
    pub _60pct_to_70pct_in_profit: T,
    pub _50pct_to_60pct_in_profit: T,
    pub _40pct_to_50pct_in_profit: T,
    pub _30pct_to_40pct_in_profit: T,
    pub _20pct_to_30pct_in_profit: T,
    pub _10pct_to_20pct_in_profit: T,
    pub _0pct_to_10pct_in_profit: T,
    pub _0pct_to_10pct_in_loss: T,
    pub _10pct_to_20pct_in_loss: T,
    pub _20pct_to_30pct_in_loss: T,
    pub _30pct_to_40pct_in_loss: T,
    pub _40pct_to_50pct_in_loss: T,
    pub _50pct_to_60pct_in_loss: T,
    pub _60pct_to_70pct_in_loss: T,
    pub _70pct_to_80pct_in_loss: T,
    pub _80pct_to_90pct_in_loss: T,
    pub _90pct_to_100pct_in_loss: T,
}

define_column_id!(
    ProfitabilityRangeId for ProfitabilityRange, version = 1 {
        Over1000PctInProfit => over_1000pct_in_profit,
        From500PctTo1000PctInProfit => _500pct_to_1000pct_in_profit,
        From300PctTo500PctInProfit => _300pct_to_500pct_in_profit,
        From200PctTo300PctInProfit => _200pct_to_300pct_in_profit,
        From100PctTo200PctInProfit => _100pct_to_200pct_in_profit,
        From90PctTo100PctInProfit => _90pct_to_100pct_in_profit,
        From80PctTo90PctInProfit => _80pct_to_90pct_in_profit,
        From70PctTo80PctInProfit => _70pct_to_80pct_in_profit,
        From60PctTo70PctInProfit => _60pct_to_70pct_in_profit,
        From50PctTo60PctInProfit => _50pct_to_60pct_in_profit,
        From40PctTo50PctInProfit => _40pct_to_50pct_in_profit,
        From30PctTo40PctInProfit => _30pct_to_40pct_in_profit,
        From20PctTo30PctInProfit => _20pct_to_30pct_in_profit,
        From10PctTo20PctInProfit => _10pct_to_20pct_in_profit,
        From0PctTo10PctInProfit => _0pct_to_10pct_in_profit,
        From0PctTo10PctInLoss => _0pct_to_10pct_in_loss,
        From10PctTo20PctInLoss => _10pct_to_20pct_in_loss,
        From20PctTo30PctInLoss => _20pct_to_30pct_in_loss,
        From30PctTo40PctInLoss => _30pct_to_40pct_in_loss,
        From40PctTo50PctInLoss => _40pct_to_50pct_in_loss,
        From50PctTo60PctInLoss => _50pct_to_60pct_in_loss,
        From60PctTo70PctInLoss => _60pct_to_70pct_in_loss,
        From70PctTo80PctInLoss => _70pct_to_80pct_in_loss,
        From80PctTo90PctInLoss => _80pct_to_90pct_in_loss,
        From90PctTo100PctInLoss => _90pct_to_100pct_in_loss,
    }
);

impl ProfitabilityRangeId {
    pub const fn is_profit(self) -> bool {
        (self as usize) < PROFIT_COUNT + 1
    }
}

/// Number of profitability range buckets.
pub const PROFITABILITY_RANGE_COUNT: usize = 25;

impl<T> ProfitabilityRange<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(&'static str) -> T,
    {
        let n = &PROFITABILITY_RANGE_NAMES;
        Self {
            over_1000pct_in_profit: create(n.over_1000pct_in_profit.id),
            _500pct_to_1000pct_in_profit: create(n._500pct_to_1000pct_in_profit.id),
            _300pct_to_500pct_in_profit: create(n._300pct_to_500pct_in_profit.id),
            _200pct_to_300pct_in_profit: create(n._200pct_to_300pct_in_profit.id),
            _100pct_to_200pct_in_profit: create(n._100pct_to_200pct_in_profit.id),
            _90pct_to_100pct_in_profit: create(n._90pct_to_100pct_in_profit.id),
            _80pct_to_90pct_in_profit: create(n._80pct_to_90pct_in_profit.id),
            _70pct_to_80pct_in_profit: create(n._70pct_to_80pct_in_profit.id),
            _60pct_to_70pct_in_profit: create(n._60pct_to_70pct_in_profit.id),
            _50pct_to_60pct_in_profit: create(n._50pct_to_60pct_in_profit.id),
            _40pct_to_50pct_in_profit: create(n._40pct_to_50pct_in_profit.id),
            _30pct_to_40pct_in_profit: create(n._30pct_to_40pct_in_profit.id),
            _20pct_to_30pct_in_profit: create(n._20pct_to_30pct_in_profit.id),
            _10pct_to_20pct_in_profit: create(n._10pct_to_20pct_in_profit.id),
            _0pct_to_10pct_in_profit: create(n._0pct_to_10pct_in_profit.id),
            _0pct_to_10pct_in_loss: create(n._0pct_to_10pct_in_loss.id),
            _10pct_to_20pct_in_loss: create(n._10pct_to_20pct_in_loss.id),
            _20pct_to_30pct_in_loss: create(n._20pct_to_30pct_in_loss.id),
            _30pct_to_40pct_in_loss: create(n._30pct_to_40pct_in_loss.id),
            _40pct_to_50pct_in_loss: create(n._40pct_to_50pct_in_loss.id),
            _50pct_to_60pct_in_loss: create(n._50pct_to_60pct_in_loss.id),
            _60pct_to_70pct_in_loss: create(n._60pct_to_70pct_in_loss.id),
            _70pct_to_80pct_in_loss: create(n._70pct_to_80pct_in_loss.id),
            _80pct_to_90pct_in_loss: create(n._80pct_to_90pct_in_loss.id),
            _90pct_to_100pct_in_loss: create(n._90pct_to_100pct_in_loss.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str) -> Result<T, E>,
    {
        let n = &PROFITABILITY_RANGE_NAMES;
        Ok(Self {
            over_1000pct_in_profit: create(n.over_1000pct_in_profit.id)?,
            _500pct_to_1000pct_in_profit: create(n._500pct_to_1000pct_in_profit.id)?,
            _300pct_to_500pct_in_profit: create(n._300pct_to_500pct_in_profit.id)?,
            _200pct_to_300pct_in_profit: create(n._200pct_to_300pct_in_profit.id)?,
            _100pct_to_200pct_in_profit: create(n._100pct_to_200pct_in_profit.id)?,
            _90pct_to_100pct_in_profit: create(n._90pct_to_100pct_in_profit.id)?,
            _80pct_to_90pct_in_profit: create(n._80pct_to_90pct_in_profit.id)?,
            _70pct_to_80pct_in_profit: create(n._70pct_to_80pct_in_profit.id)?,
            _60pct_to_70pct_in_profit: create(n._60pct_to_70pct_in_profit.id)?,
            _50pct_to_60pct_in_profit: create(n._50pct_to_60pct_in_profit.id)?,
            _40pct_to_50pct_in_profit: create(n._40pct_to_50pct_in_profit.id)?,
            _30pct_to_40pct_in_profit: create(n._30pct_to_40pct_in_profit.id)?,
            _20pct_to_30pct_in_profit: create(n._20pct_to_30pct_in_profit.id)?,
            _10pct_to_20pct_in_profit: create(n._10pct_to_20pct_in_profit.id)?,
            _0pct_to_10pct_in_profit: create(n._0pct_to_10pct_in_profit.id)?,
            _0pct_to_10pct_in_loss: create(n._0pct_to_10pct_in_loss.id)?,
            _10pct_to_20pct_in_loss: create(n._10pct_to_20pct_in_loss.id)?,
            _20pct_to_30pct_in_loss: create(n._20pct_to_30pct_in_loss.id)?,
            _30pct_to_40pct_in_loss: create(n._30pct_to_40pct_in_loss.id)?,
            _40pct_to_50pct_in_loss: create(n._40pct_to_50pct_in_loss.id)?,
            _50pct_to_60pct_in_loss: create(n._50pct_to_60pct_in_loss.id)?,
            _60pct_to_70pct_in_loss: create(n._60pct_to_70pct_in_loss.id)?,
            _70pct_to_80pct_in_loss: create(n._70pct_to_80pct_in_loss.id)?,
            _80pct_to_90pct_in_loss: create(n._80pct_to_90pct_in_loss.id)?,
            _90pct_to_100pct_in_loss: create(n._90pct_to_100pct_in_loss.id)?,
        })
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        [
            &self.over_1000pct_in_profit,
            &self._500pct_to_1000pct_in_profit,
            &self._300pct_to_500pct_in_profit,
            &self._200pct_to_300pct_in_profit,
            &self._100pct_to_200pct_in_profit,
            &self._90pct_to_100pct_in_profit,
            &self._80pct_to_90pct_in_profit,
            &self._70pct_to_80pct_in_profit,
            &self._60pct_to_70pct_in_profit,
            &self._50pct_to_60pct_in_profit,
            &self._40pct_to_50pct_in_profit,
            &self._30pct_to_40pct_in_profit,
            &self._20pct_to_30pct_in_profit,
            &self._10pct_to_20pct_in_profit,
            &self._0pct_to_10pct_in_profit,
            &self._0pct_to_10pct_in_loss,
            &self._10pct_to_20pct_in_loss,
            &self._20pct_to_30pct_in_loss,
            &self._30pct_to_40pct_in_loss,
            &self._40pct_to_50pct_in_loss,
            &self._50pct_to_60pct_in_loss,
            &self._60pct_to_70pct_in_loss,
            &self._70pct_to_80pct_in_loss,
            &self._80pct_to_90pct_in_loss,
            &self._90pct_to_100pct_in_loss,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + ExactSizeIterator {
        self.iter_mut_with_is_profit().map(|(_, v)| v)
    }

    /// Iterate mutably, yielding `(is_profit, &mut T)` for each range.
    pub fn iter_mut_with_is_profit(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (bool, &mut T)> + ExactSizeIterator {
        [
            (true, &mut self.over_1000pct_in_profit),
            (true, &mut self._500pct_to_1000pct_in_profit),
            (true, &mut self._300pct_to_500pct_in_profit),
            (true, &mut self._200pct_to_300pct_in_profit),
            (true, &mut self._100pct_to_200pct_in_profit),
            (true, &mut self._90pct_to_100pct_in_profit),
            (true, &mut self._80pct_to_90pct_in_profit),
            (true, &mut self._70pct_to_80pct_in_profit),
            (true, &mut self._60pct_to_70pct_in_profit),
            (true, &mut self._50pct_to_60pct_in_profit),
            (true, &mut self._40pct_to_50pct_in_profit),
            (true, &mut self._30pct_to_40pct_in_profit),
            (true, &mut self._20pct_to_30pct_in_profit),
            (true, &mut self._10pct_to_20pct_in_profit),
            (true, &mut self._0pct_to_10pct_in_profit),
            (false, &mut self._0pct_to_10pct_in_loss),
            (false, &mut self._10pct_to_20pct_in_loss),
            (false, &mut self._20pct_to_30pct_in_loss),
            (false, &mut self._30pct_to_40pct_in_loss),
            (false, &mut self._40pct_to_50pct_in_loss),
            (false, &mut self._50pct_to_60pct_in_loss),
            (false, &mut self._60pct_to_70pct_in_loss),
            (false, &mut self._70pct_to_80pct_in_loss),
            (false, &mut self._80pct_to_90pct_in_loss),
            (false, &mut self._90pct_to_100pct_in_loss),
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self.over_1000pct_in_profit,
            &mut self._500pct_to_1000pct_in_profit,
            &mut self._300pct_to_500pct_in_profit,
            &mut self._200pct_to_300pct_in_profit,
            &mut self._100pct_to_200pct_in_profit,
            &mut self._90pct_to_100pct_in_profit,
            &mut self._80pct_to_90pct_in_profit,
            &mut self._70pct_to_80pct_in_profit,
            &mut self._60pct_to_70pct_in_profit,
            &mut self._50pct_to_60pct_in_profit,
            &mut self._40pct_to_50pct_in_profit,
            &mut self._30pct_to_40pct_in_profit,
            &mut self._20pct_to_30pct_in_profit,
            &mut self._10pct_to_20pct_in_profit,
            &mut self._0pct_to_10pct_in_profit,
            &mut self._0pct_to_10pct_in_loss,
            &mut self._10pct_to_20pct_in_loss,
            &mut self._20pct_to_30pct_in_loss,
            &mut self._30pct_to_40pct_in_loss,
            &mut self._40pct_to_50pct_in_loss,
            &mut self._50pct_to_60pct_in_loss,
            &mut self._60pct_to_70pct_in_loss,
            &mut self._70pct_to_80pct_in_loss,
            &mut self._80pct_to_90pct_in_loss,
            &mut self._90pct_to_100pct_in_loss,
        ]
        .into_par_iter()
    }

    /// Access as a fixed-size array of references (for indexed access during merge).
    pub fn as_array(&self) -> [&T; PROFITABILITY_RANGE_COUNT] {
        [
            &self.over_1000pct_in_profit,
            &self._500pct_to_1000pct_in_profit,
            &self._300pct_to_500pct_in_profit,
            &self._200pct_to_300pct_in_profit,
            &self._100pct_to_200pct_in_profit,
            &self._90pct_to_100pct_in_profit,
            &self._80pct_to_90pct_in_profit,
            &self._70pct_to_80pct_in_profit,
            &self._60pct_to_70pct_in_profit,
            &self._50pct_to_60pct_in_profit,
            &self._40pct_to_50pct_in_profit,
            &self._30pct_to_40pct_in_profit,
            &self._20pct_to_30pct_in_profit,
            &self._10pct_to_20pct_in_profit,
            &self._0pct_to_10pct_in_profit,
            &self._0pct_to_10pct_in_loss,
            &self._10pct_to_20pct_in_loss,
            &self._20pct_to_30pct_in_loss,
            &self._30pct_to_40pct_in_loss,
            &self._40pct_to_50pct_in_loss,
            &self._50pct_to_60pct_in_loss,
            &self._60pct_to_70pct_in_loss,
            &self._70pct_to_80pct_in_loss,
            &self._80pct_to_90pct_in_loss,
            &self._90pct_to_100pct_in_loss,
        ]
    }

    /// Access as a fixed-size array of mutable references (for indexed access during merge).
    pub fn as_array_mut(&mut self) -> [&mut T; PROFITABILITY_RANGE_COUNT] {
        [
            &mut self.over_1000pct_in_profit,
            &mut self._500pct_to_1000pct_in_profit,
            &mut self._300pct_to_500pct_in_profit,
            &mut self._200pct_to_300pct_in_profit,
            &mut self._100pct_to_200pct_in_profit,
            &mut self._90pct_to_100pct_in_profit,
            &mut self._80pct_to_90pct_in_profit,
            &mut self._70pct_to_80pct_in_profit,
            &mut self._60pct_to_70pct_in_profit,
            &mut self._50pct_to_60pct_in_profit,
            &mut self._40pct_to_50pct_in_profit,
            &mut self._30pct_to_40pct_in_profit,
            &mut self._20pct_to_30pct_in_profit,
            &mut self._10pct_to_20pct_in_profit,
            &mut self._0pct_to_10pct_in_profit,
            &mut self._0pct_to_10pct_in_loss,
            &mut self._10pct_to_20pct_in_loss,
            &mut self._20pct_to_30pct_in_loss,
            &mut self._30pct_to_40pct_in_loss,
            &mut self._40pct_to_50pct_in_loss,
            &mut self._50pct_to_60pct_in_loss,
            &mut self._60pct_to_70pct_in_loss,
            &mut self._70pct_to_80pct_in_loss,
            &mut self._80pct_to_90pct_in_loss,
            &mut self._90pct_to_100pct_in_loss,
        ]
    }
}
