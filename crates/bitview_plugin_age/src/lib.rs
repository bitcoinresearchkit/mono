mod accounting_sources;
mod compute;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;
mod rhodl;
mod state;

pub use accounting_sources::AccountingSources;
pub use dependencies::Dependencies;
pub use has::HasAge;
pub use metrics::RangeVecs;

use bitview_cohort::{AgeRange, ByHalvingEpoch, Vintage};
use bitview_distribution::metrics::CreationCohort;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{CoinBlocks, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, PerBlockCumulativeRolling};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("age"), Version::new(52));
pub const ID: PluginId = STORAGE.id();

/// Age-derived metrics and resident state, independent of address state. Members first:
/// each UTXO age range, halving epoch and creation year holds its own series.
#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
    /// The capital of every age range together, each cohort's capital share's denominator.
    #[traversable(hidden)]
    all_capital: CachedSeries<Height, Cents, M>,
    /// Ages run on monotonic block time, the running maximum of block-header times.
    pub ranges: Box<AgeRange<RangeVecs<M>>>,
    /// Realized HODL (RHODL) ratio: realized cap of the 1d–1w band divided by
    /// realized cap of the 1y–2y band (1y–18m plus 18m–2y), weighted by market
    /// age in fractional days since 2009-01-03 UTC. The young band's weight is elapsed
    /// market days and the older band's weight is 1; no band-width weighting.
    /// Coin ages use monotonic block time, and 1y–2y means 365–730 days.
    /// Null when the older band's realized cap is zero.
    #[traversable(lead = "RHODL ratio: young coins' realized value against older coins', weighted by the market's age.")]
    rhodl_ratio: LazyPerBlock<Ratio>,
    halving_epochs: Box<ByHalvingEpoch<CreationCohort<M>>>,
    /// Years are UTC calendar years of the creation block's monotonic time, the running maximum of
    /// block-header times.
    vintages: Box<Vintage<CreationCohort<M>>>,
    /// Coinblocks destroyed by each block: each spent output's BTC value
    /// multiplied by the blocks it stayed unspent. Published by cointime, beside the coinblocks
    /// created and stored.
    #[traversable(hidden)]
    pub coinblocks_destroyed: PerBlockCumulativeRolling<CoinBlocks, M>,
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
