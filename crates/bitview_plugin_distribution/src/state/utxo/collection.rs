use std::path::Path;

use bitview_cohort::{
    AgeRange, AgeRangeId, AmountRange, ByEntry, ByEpoch, Class, CohortContext, CohortId,
    SpendableType, Term,
};
use brk_error::{Error, Result};
use brk_types::{Height, Sats, StoredU64};
use rayon::{prelude::*, scope as RayonScope};
use vecdb::ReadableVec;

use super::{CostBasisFenwick, UTXOCohortState, UTXOTransientState};
use crate::{
    metrics::CohortMetrics,
    state::{
        CoreRealizedState, CostBasisData, CostBasisOps, CostBasisRaw, MinimalRealizedState,
        RealizedOps, RealizedState, WithCapital, WithoutCapital,
    },
};

pub struct UTXOStates {
    pub age_range: AgeRange<UTXOCohortState<RealizedState, CostBasisData<WithCapital>>>,
    pub epoch: ByEpoch<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub class: Class<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub entry: ByEntry<UTXOCohortState<CoreRealizedState, CostBasisData<WithoutCapital>>>,
    pub amount_range: AmountRange<UTXOCohortState<MinimalRealizedState, CostBasisRaw>>,
    pub type_: SpendableType<UTXOCohortState<MinimalRealizedState, CostBasisData<WithoutCapital>>>,
    pub transient: UTXOTransientState,
}

impl UTXOStates {
    pub fn new(path: &Path) -> Self {
        let name = |id: CohortId| CohortContext::Utxo.full_name(id);

        Self {
            age_range: AgeRange::new(|id| UTXOCohortState::new(path, &name(id))),
            epoch: ByEpoch::new(|id| UTXOCohortState::new(path, &name(id))),
            class: Class::new(|id| UTXOCohortState::new(path, &name(id))),
            entry: ByEntry::new(|id| UTXOCohortState::new(path, &name(id))),
            amount_range: AmountRange::new(|id| UTXOCohortState::new(path, &name(id))),
            type_: SpendableType::new(|id| UTXOCohortState::new(path, &name(id))),
            transient: UTXOTransientState::default(),
        }
    }

    pub fn reset(&mut self) -> Result<()> {
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

    pub fn import(&mut self, metrics: &CohortMetrics, height: Height) -> Result<bool> {
        for ((state, supply), unspent_count) in self
            .age_range
            .iter_mut()
            .zip(metrics.supply.total.cohorts.utxo.age.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo.age.iter())
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
            .zip(metrics.supply.total.cohorts.utxo.epoch.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo.epoch.iter())
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
            .zip(metrics.supply.total.cohorts.utxo.class.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo.class.iter())
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
            .zip(metrics.supply.total.cohorts.utxo.entry.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo.entry.iter())
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
            .zip(metrics.supply.total.cohorts.utxo.utxo_amount.iter())
            .zip(
                metrics
                    .outputs
                    .unspent_count
                    .cohorts
                    .utxo
                    .utxo_amount
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
            .zip(metrics.supply.total.cohorts.utxo.type_.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo.type_.iter())
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

        previous_height = match state.import_at_or_before(previous_height) {
            Ok(height) => height,
            Err(Error::NotFound(_)) => return Ok(Height::ZERO),
            Err(error) => return Err(error),
        };
        state.supply.value = total_supply.collect_one(previous_height).unwrap();
        state.supply.utxo_count = *unspent_count.collect_one(previous_height).unwrap();
        state.restore_realized_cap();
        Ok(previous_height.incremented())
    }

    pub fn apply_pending(&mut self) {
        let Self {
            age_range,
            epoch,
            class,
            entry,
            type_,
            ..
        } = self;
        RayonScope(|scope| {
            for state in age_range.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in epoch.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in class.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in entry.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
            for state in type_.iter_mut() {
                scope.spawn(move |_| state.apply_pending());
            }
        });
    }

    pub fn reset_block(&mut self) {
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

    pub fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        // Each cohort owns its checkpoint directory; let all groups share the pool.
        self.age_range
            .par_iter_mut()
            .map(|state| state.write(height, cleanup))
            .chain(
                self.epoch
                    .par_iter_mut()
                    .map(|state| state.write(height, cleanup)),
            )
            .chain(
                self.class
                    .par_iter_mut()
                    .map(|state| state.write(height, cleanup)),
            )
            .chain(
                self.entry
                    .par_iter_mut()
                    .map(|state| state.write(height, cleanup)),
            )
            .chain(
                self.amount_range
                    .par_iter_mut()
                    .map(|state| state.write(height, cleanup)),
            )
            .chain(
                self.type_
                    .par_iter_mut()
                    .map(|state| state.write(height, cleanup)),
            )
            .try_for_each(|result| result)
    }

    pub fn init_fenwick_if_needed(&mut self) {
        if self.transient.fenwick.is_initialized() {
            return;
        }

        let Self {
            age_range,
            transient,
            ..
        } = self;
        transient.fenwick.compute_is_sth();
        let maps = AgeRangeId::ALL.iter().filter_map(|&id| {
            let map = id.select(age_range).cost_basis_map();
            (!map.is_empty()).then(|| (map, id.term() == Term::Sth))
        });
        transient.fenwick.bulk_init(maps);
    }

    pub fn update_fenwick_from_pending(&mut self) {
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

    pub fn fenwick(&self) -> &CostBasisFenwick {
        &self.transient.fenwick
    }
}
