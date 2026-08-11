use brk_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::CohortName;

/// "At least X% profit" threshold names (14 thresholds).
pub const PROFIT_NAMES: Profit<CohortName> = Profit {
    all: CohortName::new("utxos_in_profit", "All", "In Profit"),
    _10pct: CohortName::new("utxos_over_10pct_in_profit", ">=10%", "Over 10% in Profit"),
    _20pct: CohortName::new("utxos_over_20pct_in_profit", ">=20%", "Over 20% in Profit"),
    _30pct: CohortName::new("utxos_over_30pct_in_profit", ">=30%", "Over 30% in Profit"),
    _40pct: CohortName::new("utxos_over_40pct_in_profit", ">=40%", "Over 40% in Profit"),
    _50pct: CohortName::new("utxos_over_50pct_in_profit", ">=50%", "Over 50% in Profit"),
    _60pct: CohortName::new("utxos_over_60pct_in_profit", ">=60%", "Over 60% in Profit"),
    _70pct: CohortName::new("utxos_over_70pct_in_profit", ">=70%", "Over 70% in Profit"),
    _80pct: CohortName::new("utxos_over_80pct_in_profit", ">=80%", "Over 80% in Profit"),
    _90pct: CohortName::new("utxos_over_90pct_in_profit", ">=90%", "Over 90% in Profit"),
    _100pct: CohortName::new(
        "utxos_over_100pct_in_profit",
        ">=100%",
        "Over 100% in Profit",
    ),
    _200pct: CohortName::new(
        "utxos_over_200pct_in_profit",
        ">=200%",
        "Over 200% in Profit",
    ),
    _300pct: CohortName::new(
        "utxos_over_300pct_in_profit",
        ">=300%",
        "Over 300% in Profit",
    ),
    _500pct: CohortName::new(
        "utxos_over_500pct_in_profit",
        ">=500%",
        "Over 500% in Profit",
    ),
};

/// Number of profit thresholds.
pub const PROFIT_COUNT: usize = 14;

impl Profit<CohortName> {
    pub const fn names() -> &'static Self {
        &PROFIT_NAMES
    }
}

/// 14 "at least X% profit" aggregate thresholds.
///
/// Each is a prefix sum over the profitability ranges, from most profitable down.
#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct Profit<T> {
    pub all: T,
    pub _10pct: T,
    pub _20pct: T,
    pub _30pct: T,
    pub _40pct: T,
    pub _50pct: T,
    pub _60pct: T,
    pub _70pct: T,
    pub _80pct: T,
    pub _90pct: T,
    pub _100pct: T,
    pub _200pct: T,
    pub _300pct: T,
    pub _500pct: T,
}

define_column_id!(
    ProfitId for Profit, version = 1 {
        All => all,
        Over10Pct => _10pct,
        Over20Pct => _20pct,
        Over30Pct => _30pct,
        Over40Pct => _40pct,
        Over50Pct => _50pct,
        Over60Pct => _60pct,
        Over70Pct => _70pct,
        Over80Pct => _80pct,
        Over90Pct => _90pct,
        Over100Pct => _100pct,
        Over200Pct => _200pct,
        Over300Pct => _300pct,
        Over500Pct => _500pct,
    }
);

impl<T> Profit<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(&'static str) -> T,
    {
        let n = &PROFIT_NAMES;
        Self {
            all: create(n.all.id),
            _10pct: create(n._10pct.id),
            _20pct: create(n._20pct.id),
            _30pct: create(n._30pct.id),
            _40pct: create(n._40pct.id),
            _50pct: create(n._50pct.id),
            _60pct: create(n._60pct.id),
            _70pct: create(n._70pct.id),
            _80pct: create(n._80pct.id),
            _90pct: create(n._90pct.id),
            _100pct: create(n._100pct.id),
            _200pct: create(n._200pct.id),
            _300pct: create(n._300pct.id),
            _500pct: create(n._500pct.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str) -> Result<T, E>,
    {
        let n = &PROFIT_NAMES;
        Ok(Self {
            all: create(n.all.id)?,
            _10pct: create(n._10pct.id)?,
            _20pct: create(n._20pct.id)?,
            _30pct: create(n._30pct.id)?,
            _40pct: create(n._40pct.id)?,
            _50pct: create(n._50pct.id)?,
            _60pct: create(n._60pct.id)?,
            _70pct: create(n._70pct.id)?,
            _80pct: create(n._80pct.id)?,
            _90pct: create(n._90pct.id)?,
            _100pct: create(n._100pct.id)?,
            _200pct: create(n._200pct.id)?,
            _300pct: create(n._300pct.id)?,
            _500pct: create(n._500pct.id)?,
        })
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        [
            &self.all,
            &self._10pct,
            &self._20pct,
            &self._30pct,
            &self._40pct,
            &self._50pct,
            &self._60pct,
            &self._70pct,
            &self._80pct,
            &self._90pct,
            &self._100pct,
            &self._200pct,
            &self._300pct,
            &self._500pct,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + ExactSizeIterator {
        [
            &mut self.all,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
            &mut self._90pct,
            &mut self._100pct,
            &mut self._200pct,
            &mut self._300pct,
            &mut self._500pct,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self.all,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
            &mut self._90pct,
            &mut self._100pct,
            &mut self._200pct,
            &mut self._300pct,
            &mut self._500pct,
        ]
        .into_par_iter()
    }

    /// Access as array for indexed accumulation.
    pub fn as_array_mut(&mut self) -> [&mut T; PROFIT_COUNT] {
        [
            &mut self.all,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
            &mut self._90pct,
            &mut self._100pct,
            &mut self._200pct,
            &mut self._300pct,
            &mut self._500pct,
        ]
    }

    /// Iterate from narrowest (_500pct) to broadest (all), yielding each threshold
    /// with a growing prefix slice of `ranges` (1 range, 2 ranges, ..., PROFIT_COUNT).
    pub fn iter_mut_with_growing_prefix<'a, R>(
        &'a mut self,
        ranges: &'a [R],
    ) -> impl Iterator<Item = (&'a mut T, &'a [R])> {
        self.as_array_mut()
            .into_iter()
            .rev()
            .enumerate()
            .map(move |(n, threshold)| (threshold, &ranges[..n + 2]))
    }
}
