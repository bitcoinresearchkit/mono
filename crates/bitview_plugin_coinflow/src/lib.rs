#![allow(clippy::type_complexity)]

mod age_range;
mod aggregate;
mod compute;
mod dependencies;
mod has;
mod import;
mod model;
mod weights;

pub use dependencies::Dependencies;
pub use has::HasCoinflow;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Metrics as UrpdMetrics;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use age_range::Vecs as AgeRangeVecs;
use aggregate::Vecs as AggregateVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("coinflow"), Version::new(15));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Coinflow estimates how likely UTXOs of each age are to be spent, using
    /// observed age-specific spending rates and a fitted declining tail for
    /// ages beyond the measured ranges. An age range's mobility is its
    /// estimated probability of ever being spent.
    pub age_ranges: AgeRangeVecs<M>,
    // Cohort nodes merge the weighted aggregates (`mobile`, `immobile`) with the
    // mobility-weighted URPD statistics (`cost_basis`).
    /// Coinflow-weighted cohort metrics use mobility—an age range's estimated
    /// probability of ever being spent—to separate supply likely to move from
    /// supply unlikely to move.
    #[traversable(flatten)]
    pub aggregate: AggregateVecs<M>,
    /// Mobility-weighted UTXO price distributions.
    #[traversable(flatten)]
    pub urpd: UrpdMetrics<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
