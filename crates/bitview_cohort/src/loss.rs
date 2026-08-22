use bitview_traversable::Traversable;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use super::CohortName;

/// Names for the total loss side and eight minimum-loss thresholds.
pub const LOSS_NAMES: Loss<CohortName> = Loss {
    total: CohortName::new("utxos_in_loss", "Total", "In Loss"),
    _10pct: CohortName::new("utxos_over_10pct_in_loss", ">=10%", "Over 10% in Loss"),
    _20pct: CohortName::new("utxos_over_20pct_in_loss", ">=20%", "Over 20% in Loss"),
    _30pct: CohortName::new("utxos_over_30pct_in_loss", ">=30%", "Over 30% in Loss"),
    _40pct: CohortName::new("utxos_over_40pct_in_loss", ">=40%", "Over 40% in Loss"),
    _50pct: CohortName::new("utxos_over_50pct_in_loss", ">=50%", "Over 50% in Loss"),
    _60pct: CohortName::new("utxos_over_60pct_in_loss", ">=60%", "Over 60% in Loss"),
    _70pct: CohortName::new("utxos_over_70pct_in_loss", ">=70%", "Over 70% in Loss"),
    _80pct: CohortName::new("utxos_over_80pct_in_loss", ">=80%", "Over 80% in Loss"),
};

/// Number of loss thresholds.
pub const LOSS_COUNT: usize = 9;

impl Loss<CohortName> {
    pub const fn names() -> &'static Self {
        &LOSS_NAMES
    }
}

/// Total loss-side supply and eight "at least X% loss" aggregate thresholds.
///
/// Each is a suffix sum over the profitability ranges, from most loss-making up.
#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct Loss<T> {
    /// Uses UTXOs whose creation price is at or above the represented block's
    /// spot price.
    pub total: T,
    /// Uses UTXOs whose represented-block spot price is at least 10% below
    /// creation price.
    pub _10pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 20% below
    /// creation price.
    pub _20pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 30% below
    /// creation price.
    pub _30pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 40% below
    /// creation price.
    pub _40pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 50% below
    /// creation price.
    pub _50pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 60% below
    /// creation price.
    pub _60pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 70% below
    /// creation price.
    pub _70pct: T,
    /// Uses UTXOs whose represented-block spot price is at least 80% below
    /// creation price.
    pub _80pct: T,
}

define_column_id!(
    LossId for Loss, version = 1 {
        Total => total,
        Over10Pct => _10pct,
        Over20Pct => _20pct,
        Over30Pct => _30pct,
        Over40Pct => _40pct,
        Over50Pct => _50pct,
        Over60Pct => _60pct,
        Over70Pct => _70pct,
        Over80Pct => _80pct,
    }
);

impl<T> Loss<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(&'static str) -> T,
    {
        let n = &LOSS_NAMES;
        Self {
            total: create(n.total.id),
            _10pct: create(n._10pct.id),
            _20pct: create(n._20pct.id),
            _30pct: create(n._30pct.id),
            _40pct: create(n._40pct.id),
            _50pct: create(n._50pct.id),
            _60pct: create(n._60pct.id),
            _70pct: create(n._70pct.id),
            _80pct: create(n._80pct.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str) -> Result<T, E>,
    {
        let n = &LOSS_NAMES;
        Ok(Self {
            total: create(n.total.id)?,
            _10pct: create(n._10pct.id)?,
            _20pct: create(n._20pct.id)?,
            _30pct: create(n._30pct.id)?,
            _40pct: create(n._40pct.id)?,
            _50pct: create(n._50pct.id)?,
            _60pct: create(n._60pct.id)?,
            _70pct: create(n._70pct.id)?,
            _80pct: create(n._80pct.id)?,
        })
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        [
            &self.total,
            &self._10pct,
            &self._20pct,
            &self._30pct,
            &self._40pct,
            &self._50pct,
            &self._60pct,
            &self._70pct,
            &self._80pct,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + ExactSizeIterator {
        [
            &mut self.total,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self.total,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
        ]
        .into_par_iter()
    }

    /// Access as array for indexed accumulation.
    pub fn as_array_mut(&mut self) -> [&mut T; LOSS_COUNT] {
        [
            &mut self.total,
            &mut self._10pct,
            &mut self._20pct,
            &mut self._30pct,
            &mut self._40pct,
            &mut self._50pct,
            &mut self._60pct,
            &mut self._70pct,
            &mut self._80pct,
        ]
    }

    /// Iterate from narrowest (_80pct) to broadest (total), yielding each threshold
    /// with a growing suffix slice of `ranges` (2 ranges through all loss ranges).
    pub fn iter_mut_with_growing_suffix<'a, R>(
        &'a mut self,
        ranges: &'a [R],
    ) -> impl Iterator<Item = (&'a mut T, &'a [R])> {
        let len = ranges.len();
        self.as_array_mut()
            .into_iter()
            .rev()
            .enumerate()
            .map(move |(n, threshold)| (threshold, &ranges[len - 2 - n..]))
    }
}
