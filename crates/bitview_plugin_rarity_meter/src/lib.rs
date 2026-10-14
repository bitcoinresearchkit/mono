#![allow(clippy::type_complexity)]

mod band;
mod block_decay_percentiles;
mod component;
mod component_price;
mod components;
mod compute;
mod dependencies;
mod extreme;
mod extremes;
mod import;
mod inner;
mod median_component;
mod threshold_vecs;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use band::Band;
use block_decay_percentiles::{BlockDecayPercentiles, START_HEIGHT};
use component::Component;
use components::Components;
use extremes::Extremes;
use inner::RarityMeterInner;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("rarity_meter"), Version::new(18));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Completed reference prices are supplied by Holders, Cointime and Coinflow.
    /// Reference-price components used by the Rarity Meter. A UTXO's creation
    /// price is Bitcoin's spot price when that output was created. Realized
    /// price is `sum(creation price x unspent sats) / sum(unspent sats)`;
    /// capitalized price instead weights by creation-date value and is
    /// `sum(creation price squared x unspent sats) / sum(creation price x
    /// unspent sats)`.
    components: Components<M>,
    extremes: Extremes<M>,
    /// Full Rarity Meter: the local and cycle meters' 16 reference models together,
    /// showing how unusual spot price is for any of them.
    full: RarityMeterInner<M>,
    /// Local Rarity Meter, young-coin positioning: the short-term holders' and
    /// under-six-month cohorts' realized and capitalized prices and the short-term
    /// holders' BTC- and USD-weighted median creation prices.
    local: RarityMeterInner<M>,
    /// Cycle Rarity Meter, long-cycle valuation: the all-chain realized and
    /// capitalized prices, the all-chain and mobile supply's BTC- and USD-weighted median
    /// creation prices, the vaulted price, True Market Mean, and the awake and mobile
    /// realized prices.
    cycle: RarityMeterInner<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
