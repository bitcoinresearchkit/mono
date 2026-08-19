#![allow(clippy::type_complexity)]

mod activity;
mod adjusted;
mod age_range;
mod aggregate;
mod cap;
mod dependencies;
mod prices;
mod reserve_risk;
mod supply;
mod value;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

use activity::Vecs as ActivityVecs;
use adjusted::Vecs as AdjustedVecs;
use age_range::Vecs as AgeRangeVecs;
use aggregate::Vecs as AggregateVecs;
use cap::Vecs as CapVecs;
pub use dependencies::Dependencies;
use prices::Vecs as PricesVecs;
use reserve_risk::Vecs as ReserveRiskVecs;
use supply::Vecs as SupplyVecs;
use value::Vecs as ValueVecs;

pub const ID: PluginId = PluginId::new("cointime");
const DB_NAME: &str = ID.as_str();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub activity: ActivityVecs<M>,
    pub age_range: AgeRangeVecs<M>,
    #[traversable(flatten)]
    pub aggregate: AggregateVecs<M>,
    pub supply: SupplyVecs<M>,
    pub value: ValueVecs<M>,
    pub cap: CapVecs<M>,
    pub prices: PricesVecs<M>,
    pub adjusted: AdjustedVecs<M>,
    pub reserve_risk: ReserveRiskVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
