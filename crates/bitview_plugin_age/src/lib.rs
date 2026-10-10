mod accounting_sources;
mod compute;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;
mod state;

pub use accounting_sources::AccountingSources;
pub use dependencies::Dependencies;
pub use has::HasAge;
pub use metrics::{CohortVecs, RangeVecs};

use bitview_cohort::{AgeRange, ByEpoch, Class};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::CoinBlocks;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, PerBlockCumulativeRolling};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("age"), Version::new(48));
pub const ID: PluginId = STORAGE.id();

/// Age-derived metrics and resident state, independent of address state. Members first:
/// each UTXO age range, halving epoch and creation year holds its own series.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
    /// The capital of every age range together, each cohort's capital share's denominator.
    #[traversable(hidden)]
    all_capital: CachedSeries<Height, Cents, M>,
    pub ranges: Box<AgeRange<RangeVecs<M>>>,
    epochs: Box<ByEpoch<CohortVecs<M>>>,
    classes: Box<Class<CohortVecs<M>>>,
    /// Coinblocks destroyed by each block: each spent output's BTC value
    /// multiplied by the blocks it stayed unspent.
    pub coinblocks_destroyed: PerBlockCumulativeRolling<CoinBlocks, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl<M: StorageMode> Vecs<M> {
    /// The total supply each cohort's share divides.
    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }

    /// The capital of every age range together: all unspent outputs at their creation price.
    pub fn all_capital(&self) -> &CachedSeries<Height, Cents, M> {
        &self.all_capital
    }

    /// Each age range's supply in sats.
    pub fn age_supplies(&self) -> AgeRange<&impl ReadableVec<Height, Sats>> {
        AgeRange::from_fn(|id| &id.select(&self.ranges).supply.total.value.sats.height)
    }
}
