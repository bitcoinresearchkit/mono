use crate::unrealized_data::UnrealizedData;
use crate::{
    activity::Activity, columns::Columns, cost_basis::CostBasis, data::Data, outputs::Outputs,
    ratios::Ratios, realized::Realized, relative::Relative, supply::Supply, unrealized::Unrealized,
};
use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

/// One complete metric layout, instantiated identically for every age filter.
#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    pub supply: Supply,
    pub(crate) outputs: Outputs,
    pub activity: Activity,
    pub realized: Realized,
    pub(crate) unrealized: Unrealized,
    pub cost_basis: CostBasis,
    pub ratios: Ratios<M>,
    pub relative: Relative<M>,
    #[traversable(hidden)]
    pub(crate) columns: Columns<M>,
}
impl Metrics {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        cost_basis: CostBasis,
    ) -> Result<Self> {
        let columns = Columns::import(db, id, v)?;
        let supply = Supply::new(id, v, &columns, mappings, windows, spot);
        let outputs = Outputs::new(id, v, &columns, mappings, windows);
        let activity = Activity::new(id, v, &columns, mappings, windows);
        let realized = Realized::new(id, v, &columns, mappings, windows, spot);
        let relative = Relative::import(db, id, v, mappings)?;
        let unrealized = Unrealized::new(id, v, &columns, mappings);
        Ok(Self {
            supply,
            outputs,
            activity,
            realized,
            unrealized,
            cost_basis,
            ratios: Ratios::import(db, id, v, mappings, &columns, windows)?,
            relative,
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
        exit: &Exit,
    ) -> Result<()> {
        self.ratios
            .compute(from, &self.activity, &self.realized, exit)?;
        self.compute_relative(from, all_supply, all_market_cap, exit)
    }
    pub(crate) fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.columns.stored_vecs_mut()
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut v = self.columns.stored_vecs_mut();
        v.extend(self.ratios.stored_vecs_mut());
        v.extend(self.relative.stored_vecs_mut());
        v
    }
}
