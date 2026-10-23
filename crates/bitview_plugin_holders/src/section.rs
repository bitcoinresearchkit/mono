use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_vecs::{CachedSeries, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec};

use crate::{activity::Activity, columns::Columns, cost_basis::CostBasisVecs};

/// What the holder groups import from.
pub(crate) struct Imports<'a> {
    pub db: &'a Database,
    pub v: Version,
    pub columns: &'a AgeAggregate<Columns>,
    pub mappings: &'a Mappings,
    pub windows: &'a Windows<&'a LazyWindowStartVec>,
    pub spot: &'a ReadableBoxedVec<Height, Cents>,
    pub cost_basis: &'a CostBasisVecs,
    /// The under-an-hour band's cumulative transfer volume in cents, which aSOPR leaves out.
    pub under_1h_volume: &'a CachedSeries<Height, Cents>,
    /// The under-an-hour band's cumulative value destroyed, which aSOPR leaves out.
    pub under_1h_value_destroyed: &'a CachedSeries<Height, Cents>,
}

/// What the holder groups compute from after the block loop.
pub(crate) struct Rest<'a, S> {
    pub from: Height,
    pub columns: &'a AgeAggregate<Columns>,
    pub all_supply: &'a ReadableBoxedVec<Height, Sats>,
    pub all_capital: &'a ReadableBoxedVec<Height, Cents>,
    pub spot: &'a S,
    pub exit: &'a Exit,
}

/// What a group's sections compute from after the block loop.
pub(crate) struct CohortCompute<'a> {
    pub from: Height,
    pub c: &'a Columns,
    pub all_supply: &'a ReadableBoxedVec<Height, Sats>,
    pub all_capital: &'a ReadableBoxedVec<Height, Cents>,
    pub activity: &'a Activity,
    pub exit: &'a Exit,
}

/// A section of a group's series whose shape differs by group: its supply, capital and realized
/// flows.
pub(crate) trait Section: Sized {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self>;
    fn compute(&mut self, cx: &CohortCompute) -> Result<()>;
}
