#![allow(clippy::type_complexity)]

mod activity;
mod adjusted;
mod age_range;
mod aggregate;
mod cap;
mod dependencies;
mod has;
mod prices;
mod reserve_risk;
mod supply;
mod value;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use activity::Vecs as ActivityVecs;
use adjusted::Vecs as AdjustedVecs;
use age_range::Vecs as AgeRangeVecs;
use aggregate::Vecs as AggregateVecs;
use cap::Vecs as CapVecs;
pub use dependencies::Dependencies;
pub use has::HasCointime;
use prices::Vecs as PricesVecs;
use reserve_risk::Vecs as ReserveRiskVecs;
use supply::Vecs as SupplyVecs;
use value::Vecs as ValueVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("cointime"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    /// Cointime measures how long bitcoin remains unspent. One coinblock is one
    /// BTC held for one block interval; spending destroys the coinblocks that
    /// the spent outputs accumulated.
    pub activity: ActivityVecs<M>,
    /// Age-range cointime allocates creation and destruction of coin days to the
    /// UTXO ages where they occur. One coin day is one BTC held unspent for one
    /// day. An age range's wakefulness is the share of its cumulatively created
    /// coin days that spending has consumed; one minus wakefulness is the share
    /// still stored.
    pub age_range: AgeRangeVecs<M>,
    #[traversable(flatten)]
    /// Cointime-weighted cohort metrics use wakefulness—the share of an age
    /// range's accumulated coin days that has been consumed—to separate more
    /// economically active supply from more dormant supply.
    pub aggregate: AggregateVecs<M>,
    /// Cointime's active and vaulted supply estimates split circulating supply
    /// using liveliness, the cumulative share of created coinblocks that has
    /// been destroyed.
    pub supply: SupplyVecs<M>,
    /// Cointime value metrics assign the represented block's spot price to
    /// coinblocks created, destroyed, or stored. One coinblock is one BTC held
    /// for one block interval; spending destroys the coinblocks accumulated by
    /// the spent outputs.
    pub value: ValueVecs<M>,
    /// Cointime capitalization metrics reweight realized capitalization—the
    /// sum of each unspent output's BTC value at Bitcoin's spot price when it
    /// was created—or cumulative on-chain value by economic activity and
    /// dormancy.
    pub cap: CapVecs<M>,
    /// Cointime reference prices translate activity-adjusted capitalization or
    /// value into a price per BTC. They are model-derived benchmarks, not traded
    /// market prices.
    pub prices: PricesVecs<M>,
    /// Cointime-adjusted rates multiply a conventional rate by the ratio of
    /// active to vaulted supply, `liveliness / (1 - liveliness)`.
    pub adjusted: AdjustedVecs<M>,
    /// Reserve Risk compares spot price with the cumulative opportunity cost of
    /// holders not spending older coins; lower values mean price is low relative
    /// to that accumulated holder reserve.
    pub reserve_risk: ReserveRiskVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
