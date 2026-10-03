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

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("rarity_meter"), Version::new(17));
pub const ID: PluginId = STORAGE.id();
const COMPUTE_BATCH_SIZE: usize = 100_000;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Completed reference prices are supplied by Aggregated, Cointime and Coinflow.
    /// Reference-price components used by the Rarity Meter. A UTXO's creation
    /// price is Bitcoin's spot price when that output was created. Realized
    /// price is `sum(creation price x unspent sats) / sum(unspent sats)`;
    /// capitalized price instead weights by creation-date value and is
    /// `sum(creation price squared x unspent sats) / sum(creation price x
    /// unspent sats)`.
    components: Components<M>,
    extremes: Extremes<M>,
    /// Full Rarity Meter combining local and cycle views to show how unusual
    /// spot price is across both young-coin and long-cycle reference models.
    full: RarityMeterInner<M>,
    /// Full V2 combines 24 reference-price components, excluding LTH realized,
    /// capitalized, and median prices, with the three lower-only Bedrock models.
    full_v2: RarityMeterInner<M>,
    /// Local Rarity Meter focused on young-coin positioning. It combines
    /// under-four-month and under-six-month realized price with short-term-holder
    /// realized and capitalized price.
    local: RarityMeterInner<M>,
    /// Local V2 adds under-four-month and under-six-month capitalized prices
    /// and BTC- and USD-weighted STH median prices to Local's four reference models.
    local_v2: RarityMeterInner<M>,
    /// Cycle Rarity Meter focused on long-cycle valuation. It combines six
    /// old-coin and all-chain reference-price models with rare lower-price
    /// boundaries from the raw, cointime, and coinflow Bedrock models.
    cycle: RarityMeterInner<M>,
    /// Cycle V2 combines 16 all-chain, older-coin, cointime, and coinflow
    /// reference prices with three Bedrock floors, excluding LTH-specific prices.
    cycle_v2: RarityMeterInner<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
