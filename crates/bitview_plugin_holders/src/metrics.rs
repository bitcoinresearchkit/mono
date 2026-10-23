use bitview_cohort::{AgeAggregateId, BuildMember, UpdateMember};
use bitview_distribution::metrics::UtxoViews;
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{Cents, Height};
use vecdb::{ReadableVec, Rw, StorageMode};

use crate::{
    activity::Activity,
    adjusted_sopr::AdjustedRealized,
    capital::{Capital, SharedCapital},
    cost_basis::CostBasis,
    realized::Realized,
    section::{CohortCompute, Imports, Rest, Section},
    supply::{SharedSupply, Supply},
    unrealized::Unrealized,
};

/// The sections that differ by holder group: what [`Metrics`] holds as its supply, capital and
/// realized flows.
pub trait Kind {
    type Supply<M: StorageMode>;
    type Capital<M: StorageMode>;
    type Realized<M: StorageMode>;
}

/// All coins: no shares of all (100% throughout).
pub struct All;
/// The groups holding outputs under an hour old but not all (`sth`, `under_4m`, `under_6m`):
/// shares of all and aSOPR.
pub struct Young;
/// The groups holding no output under an hour old (`lth`, `over_4m`, `over_6m`): shares of all,
/// no aSOPR (it equals their SOPR).
pub struct Old;

impl Kind for All {
    type Supply<M: StorageMode> = Supply<M>;
    type Capital<M: StorageMode> = Capital<M>;
    type Realized<M: StorageMode> = AdjustedRealized<M>;
}
impl Kind for Young {
    type Supply<M: StorageMode> = SharedSupply<M>;
    type Capital<M: StorageMode> = SharedCapital<M>;
    type Realized<M: StorageMode> = AdjustedRealized<M>;
}
impl Kind for Old {
    type Supply<M: StorageMode> = SharedSupply<M>;
    type Capital<M: StorageMode> = SharedCapital<M>;
    type Realized<M: StorageMode> = Realized<M>;
}

/// A holder group's series, its sections by its [`Kind`].
#[derive(Traversable)]
pub struct Metrics<K: Kind, M: StorageMode = Rw> {
    pub supply: K::Supply<M>,
    pub capital: K::Capital<M>,
    utxos: UtxoViews,
    pub activity: Activity<M>,
    pub realized: K::Realized<M>,
    unrealized: Unrealized,
    pub cost_basis: CostBasis<M>,
}

impl<K: Kind> BuildMember<Imports<'_>> for Metrics<K>
where
    K::Supply<Rw>: Section,
    K::Capital<Rw>: Section,
    K::Realized<Rw>: Section,
{
    type Error = Error;
    fn build(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        let Imports {
            db,
            v,
            mappings,
            windows,
            cost_basis,
            ..
        } = *cx;
        let c = id.select(cx.columns);
        Ok(Self {
            supply: Section::import(id, cx)?,
            capital: Section::import(id, cx)?,
            utxos: UtxoViews::new(
                |metric| id.metric_name(metric),
                v,
                v,
                &c.count,
                &c.spent_count,
                mappings,
                windows,
            ),
            activity: Activity::import(db, id, v, c, mappings, windows)?,
            realized: Section::import(id, cx)?,
            unrealized: Unrealized::new(id, v, c, mappings),
            cost_basis: CostBasis::import(db, id, v, c, mappings, cost_basis)?,
        })
    }
}

impl<K: Kind, S: ReadableVec<Height, Cents>> UpdateMember<Rest<'_, S>> for Metrics<K>
where
    K::Supply<Rw>: Section,
    K::Capital<Rw>: Section,
    K::Realized<Rw>: Section,
{
    type Error = Error;
    /// Derives the group's series from its columns after the block loop.
    fn update(&mut self, id: AgeAggregateId, rest: &Rest<S>) -> Result<()> {
        let Rest {
            from,
            columns,
            all_supply,
            all_capital,
            spot,
            exit,
        } = *rest;
        let Self {
            supply,
            capital,
            activity,
            realized,
            cost_basis,
            ..
        } = self;
        activity.compute(from, exit)?;
        let cx = CohortCompute {
            from,
            c: id.select(columns),
            all_supply,
            all_capital,
            activity,
            exit,
        };
        supply.compute(&cx)?;
        capital.compute(&cx)?;
        realized.compute(&cx)?;
        cost_basis.compute(from, spot, exit)
    }
}
