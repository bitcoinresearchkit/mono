use bitview_cohort::AgeAggregateId;
use bitview_distribution::metrics::OutputViews;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

use crate::{
    activity::Activity,
    capital::Capital,
    columns::Columns,
    cost_basis::{CostBasis, CostBasisVecs},
    realized::Realized,
    supply::Supply,
    unrealized::Unrealized,
};

#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    pub supply: Supply<M>,
    pub capital: Capital<M>,
    outputs: OutputViews,
    pub activity: Activity<M>,
    pub realized: Realized<M>,
    unrealized: Unrealized,
    pub cost_basis: CostBasis<M>,
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
        sources: &CostBasisVecs,
    ) -> Result<Self> {
        let columns = Columns::import(db, id, v)?;
        let c = &columns;
        let cost_basis = CostBasis::import(db, id, v, c, mappings, sources)?;
        Ok(Self {
            supply: Supply::import(db, id, v, c, mappings, windows, spot)?,
            capital: Capital::import(db, id, v, c, mappings, windows)?,
            outputs: OutputViews::new(
                |metric| id.metric_name(metric),
                v,
                v,
                &c.count,
                &c.spent_count,
                mappings,
                windows,
            ),
            activity: Activity::import(db, id, v, c, mappings, windows)?,
            realized: Realized::import(db, id, v, c, mappings, windows)?,
            unrealized: Unrealized::new(id, v, c, mappings),
            cost_basis,
            columns,
        })
    }
    pub(crate) fn compute_rest(
        &mut self,
        from: Height,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_capital: &ReadableBoxedVec<Height, Cents>,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            supply,
            capital,
            activity,
            realized,
            cost_basis,
            columns: c,
            ..
        } = self;
        supply.compute(from, c, all_supply, exit)?;
        capital.compute(from, c, all_capital, exit)?;
        activity.compute(from, exit)?;
        realized.compute(from, c, activity, exit)?;
        cost_basis.compute(from, spot, exit)
    }
    /// The columns the block loop writes.
    pub(crate) fn state_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.columns.iter_any_stored_mut()
    }
    /// The outputs [`Self::compute_rest`] derives from the columns.
    pub(crate) fn derived_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply
            .iter_any_stored_mut()
            .chain(self.capital.iter_any_stored_mut())
            .chain(self.activity.iter_any_stored_mut())
            .chain(self.realized.iter_any_stored_mut())
            .chain(self.cost_basis.iter_any_stored_mut())
    }
}
