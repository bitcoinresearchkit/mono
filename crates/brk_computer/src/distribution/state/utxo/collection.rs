use std::path::Path;

use brk_cohort::{
    AgeRange, AgeRangeId, AmountRange, ByEntry, ByEpoch, Class, CohortContext, Filter,
    SpendableType,
};
use brk_error::Result;
use brk_types::{Height, Sats, StoredU64};
use rayon::prelude::*;
use vecdb::{ColumnId, ReadableVec};

use super::{CostBasisFenwick, UTXOCohortState, UTXOTransientState};
use crate::distribution::metrics::CohortMetrics;
use crate::distribution::state::{
    CoreRealizedState, CostBasisData, CostBasisOps, CostBasisRaw, MinimalRealizedState,
    RealizedOps, RealizedState, WithCapital, WithoutCapital,
};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState, CostBasisData<WithCapital>>>,
    pub epoch: ByEpoch<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub class: Class<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub entry: ByEntry<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub amount_range: AmountRange<UTXOCohortState<MinimalRealizedState, CostBasisRaw>>,
    pub type_: SpendableType<UTXOCohortState<MinimalRealizedState, CostBasisData<WithoutCapital>>>,
    pub(super) transient: UTXOTransientState,
}

impl UTXOStates {
    pub(crate) fn new(path: &Path) -> Self {
        let name = |filter: &Filter, cohort: &str| CohortContext::Utxo.full_name(filter, cohort);

        Self {
            age_range: AgeRange::new(|filter, cohort| {
                UTXOCohortState::new(path, &name(&filter, cohort))
            }),
            epoch: ByEpoch::new(|filter, cohort| {
                UTXOCohortState::new(path, &name(&filter, cohort))
            }),
            class: Class::new(|filter, cohort| UTXOCohortState::new(path, &name(&filter, cohort))),
            entry: ByEntry::new(|filter, cohort| {
                UTXOCohortState::new(path, &name(&filter, cohort))
            }),
            amount_range: AmountRange::new(|filter, cohort| {
                UTXOCohortState::new(path, &name(&filter, cohort))
            }),
            type_: SpendableType::new(|filter, cohort| {
                UTXOCohortState::new(path, &name(&filter, cohort))
            }),
            transient: UTXOTransientState::default(),
        }
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        for state in self.age_range.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        for state in self.epoch.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        for state in self.class.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        for state in self.entry.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        for state in self.amount_range.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        for state in self.type_.iter_mut() {
            state.reset();
            state.reset_cost_basis_data_if_needed()?;
        }
        self.transient = UTXOTransientState::default();
        Ok(())
    }

    pub(crate) fn import(&mut self, metrics: &CohortMetrics, height: Height) -> Result<bool> {
        for ((state, supply), unspent_count) in self
            .age_range
            .iter_mut()
            .zip(metrics.supply.total.cohorts.age.range.iter())
            .zip(metrics.outputs.unspent_count.cohorts.age.range.iter())
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        for ((state, supply), unspent_count) in self
            .epoch
            .iter_mut()
            .zip(metrics.supply.total.cohorts.epoch.iter())
            .zip(metrics.outputs.unspent_count.cohorts.epoch.iter())
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        for ((state, supply), unspent_count) in self
            .class
            .iter_mut()
            .zip(metrics.supply.total.cohorts.class.iter())
            .zip(metrics.outputs.unspent_count.cohorts.class.iter())
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        for ((state, supply), unspent_count) in self
            .entry
            .iter_mut()
            .zip(metrics.supply.total.cohorts.entry.iter())
            .zip(metrics.outputs.unspent_count.cohorts.entry.iter())
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        for ((state, supply), unspent_count) in self
            .amount_range
            .iter_mut()
            .zip(metrics.supply.total.cohorts.utxo_amount.range.iter())
            .zip(
                metrics
                    .outputs
                    .unspent_count
                    .cohorts
                    .utxo_amount
                    .range
                    .iter(),
            )
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        for ((state, supply), unspent_count) in self
            .type_
            .iter_mut()
            .zip(metrics.supply.total.cohorts.type_.iter())
            .zip(metrics.outputs.unspent_count.cohorts.type_.iter())
        {
            if Self::import_one(state, &supply.sats.height, &unspent_count.height, height)?
                != height
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn import_one<R, C>(
        state: &mut UTXOCohortState<R, C>,
        total_supply: &impl ReadableVec<Height, Sats>,
        unspent_count: &impl ReadableVec<Height, StoredU64>,
        height: Height,
    ) -> Result<Height>
    where
        R: RealizedOps,
        C: CostBasisOps,
    {
        let Some(mut previous_height) = height.decremented() else {
            return Ok(Height::ZERO);
        };

        previous_height = state.import_at_or_before(previous_height)?;
        state.supply.value = total_supply.collect_one(previous_height).unwrap();
        state.supply.utxo_count = *unspent_count.collect_one(previous_height).unwrap();
        state.restore_realized_cap();
        Ok(previous_height.incremented())
    }

    pub(crate) fn apply_pending(&mut self) {
        self.age_range
            .iter_mut()
            .for_each(|state| state.apply_pending());
        self.epoch
            .iter_mut()
            .for_each(|state| state.apply_pending());
        self.class
            .iter_mut()
            .for_each(|state| state.apply_pending());
        self.entry
            .iter_mut()
            .for_each(|state| state.apply_pending());
        self.type_
            .iter_mut()
            .for_each(|state| state.apply_pending());
    }

    pub(crate) fn reset_block(&mut self) {
        self.age_range
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.epoch
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.class
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.entry
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.amount_range
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        self.type_
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
    }

    pub(crate) fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.age_range
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        self.epoch
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        self.class
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        self.entry
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        self.amount_range
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        self.type_
            .par_iter_mut()
            .try_for_each(|state| state.write(height, cleanup))?;
        Ok(())
    }

    pub(crate) fn init_fenwick_if_needed(&mut self, sth_filter: &Filter) {
        if self.transient.fenwick.is_initialized() {
            return;
        }

        self.transient.fenwick.compute_is_sth(sth_filter);
        let maps: Vec<_> = AgeRangeId::ALL
            .iter()
            .filter_map(|&id| {
                let map = id.select(&self.age_range).cost_basis_map();
                (!map.is_empty()).then(|| (map, sth_filter.includes(id.filter())))
            })
            .collect();
        self.transient.fenwick.bulk_init(maps.into_iter());
    }

    pub(crate) fn update_fenwick_from_pending(&mut self) {
        if !self.transient.fenwick.is_initialized() {
            return;
        }

        let Self {
            age_range,
            transient,
            ..
        } = self;
        for &id in AgeRangeId::ALL {
            let is_sth = transient.fenwick.is_sth(id);
            id.select(age_range)
                .for_each_cost_basis_pending(|&price, delta| {
                    transient.fenwick.apply_delta(price, delta, is_sth);
                });
        }
    }

    pub(crate) fn fenwick(&self) -> &CostBasisFenwick {
        &self.transient.fenwick
    }
}
