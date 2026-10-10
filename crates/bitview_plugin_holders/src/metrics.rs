use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

use crate::{
    activity::Activity,
    columns::Columns,
    cost_basis::{CostBasis, CostBasisVecs},
    data::Data,
    outputs::Outputs,
    realized::Realized,
    supply::Supply,
    unrealized::Unrealized,
    unrealized_data::UnrealizedData,
};

/// One complete metric layout, instantiated identically for every age filter.
#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    pub supply: Supply<M>,
    outputs: Outputs,
    pub activity: Activity<M>,
    pub realized: Realized<M>,
    unrealized: Unrealized<M>,
    pub cost_basis: CostBasis<M>,
    #[traversable(hidden)]
    columns: Columns<M>,
}
impl Metrics {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        sources: &CostBasisVecs,
    ) -> Result<Self> {
        let columns = Columns::import(db, id, v)?;
        let c = &columns;
        let cost_basis = CostBasis::import(db, id, v, c, mappings, sources)?;
        Ok(Self {
            supply: Supply::import(db, id, v, c, mappings, windows, spot)?,
            outputs: Outputs::new(id, v, c, mappings, windows),
            activity: Activity::import(db, id, v, c, mappings, windows)?,
            realized: Realized::import(db, id, v, c, mappings, windows)?,
            unrealized: Unrealized::import(db, id, v, c, mappings)?,
            cost_basis,
            columns,
        })
    }
    pub(crate) fn push(&mut self, data: &Data, unrealized: &UnrealizedData) {
        self.columns.push(data, unrealized);
    }
    pub(crate) fn compute_rest(
        &mut self,
        from: Height,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            supply,
            activity,
            realized,
            unrealized,
            cost_basis,
            columns: c,
            ..
        } = self;
        supply.compute(from, c, all_supply, exit)?;
        activity.compute(from, exit)?;
        realized.compute(from, c, activity, all_market_cap, exit)?;
        unrealized.compute(from, c, supply, all_market_cap, exit)?;
        cost_basis.compute(from, spot, exit)
    }
    pub(crate) fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.columns.stored_vecs_mut()
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut v = self.columns.stored_vecs_mut();
        v.extend(self.supply.stored_vecs_mut());
        v.extend(self.activity.stored_vecs_mut());
        v.extend(self.realized.stored_vecs_mut());
        v.extend(self.unrealized.stored_vecs_mut());
        v.extend(self.cost_basis.stored_vecs_mut());
        v
    }
}
