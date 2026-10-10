#![allow(clippy::type_complexity)]

mod activity;
mod adjusted;
mod age_range;
mod aggregate;
mod cap;
mod compute;
mod dependencies;
mod has;
mod import;
mod prices;
mod reserve_risk;
mod supply;
mod value;
mod weights;

pub use dependencies::Dependencies;
pub use has::HasCointime;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Replay;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use activity::Vecs as ActivityVecs;
use adjusted::Vecs as AdjustedVecs;
use age_range::Vecs as AgeRangeVecs;
use aggregate::Vecs as AggregateVecs;
use cap::Vecs as CapVecs;
use prices::Vecs as PricesVecs;
use reserve_risk::Vecs as ReserveRiskVecs;
use supply::Vecs as SupplyVecs;
use value::Vecs as ValueVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("cointime"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    urpd_replay: M::WriteOnly<Replay>,

    /// Cointime measures how long bitcoin remains unspent. One coinblock is one
    /// BTC held for one block interval; spending destroys the coinblocks that
    /// the spent outputs accumulated.
    activity: ActivityVecs<M>,
    /// Age-range cointime allocates creation and destruction of coin days to the
    /// UTXO ages where they occur. One coin day is one BTC held unspent for one
    /// day. An age range's wakefulness is the share of its cumulatively created
    /// coin days that spending has consumed; one minus wakefulness is the share
    /// still stored.
    pub age_ranges: AgeRangeVecs<M>,
    /// Cointime-weighted cohort metrics use wakefulness—the share of an age
    /// range's accumulated coin days that has been consumed—to separate more
    /// economically active supply from more dormant supply.
    #[traversable(flatten)]
    pub aggregate: AggregateVecs<M>,
    /// Cointime's active and vaulted supply estimates split circulating supply
    /// using liveliness, the cumulative share of created coinblocks that has
    /// been destroyed.
    supply: SupplyVecs,
    /// Cointime value metrics assign the represented block's spot price to
    /// coinblocks created, destroyed, or stored. One coinblock is one BTC held
    /// for one block interval; spending destroys the coinblocks accumulated by
    /// the spent outputs.
    value: ValueVecs<M>,
    /// Cointime capitalizations split Bitcoin's capitalization by economic
    /// activity: market capitalization into active and vaulted, and realized
    /// capitalization (each unspent output's BTC value at Bitcoin's spot price
    /// when it was created) into investor and thermo. The Cointime
    /// capitalization values the supply from cumulative destroyed value.
    caps: CapVecs<M>,
    /// Cointime reference prices translate activity-adjusted capitalization or
    /// value into a price per BTC, each beside spot's ratio to it. They are
    /// model-derived benchmarks, not traded market prices.
    pub prices: PricesVecs<M>,
    /// Cointime-adjusted rates reweight conventional supply rates by
    /// liveliness, the share of created coinblocks that has been destroyed.
    adjusted: AdjustedVecs<M>,
    /// Reserve Risk compares spot price with the cumulative opportunity cost of
    /// holders not spending older coins; lower values mean price is low relative
    /// to that accumulated holder reserve.
    reserve_risk: ReserveRiskVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
