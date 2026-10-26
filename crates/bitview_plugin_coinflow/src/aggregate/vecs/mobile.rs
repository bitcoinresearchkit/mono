use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{LazyFiatPerBlock, LazyPerBlock, LazySpotValuePerBlock};
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct MobileVecs<M: StorageMode = Rw> {
    /// Sum of supply multiplied by mobility across the cohort's UTXO age
    /// ranges, rounded once to the nearest satoshi.
    #[traversable(wrap = "supply", rename = "total")]
    #[traversable(lead = "Mobile supply: the coins, each age weighted by how likely it is to be spent.")]
    pub supply: LazySpotValuePerBlock,
    /// Share of mobile supply that is in loss: the sum of supply in loss
    /// multiplied by mobility divided by the sum of total supply multiplied by
    /// mobility. Null when the weighted supply is zero. As a percentage.
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    #[traversable(lead = "Mobile supply in loss: the part of it now below its cost.")]
    pub supply_in_loss_share: LazyPerBlock<Percent, BoundedRatio>,
    /// Sum of creation-date USD value multiplied by mobility across the
    /// cohort's UTXO age ranges: the realized capitalization of the mobile
    /// supply. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    #[traversable(wrap = "capital", rename = "total")]
    #[traversable(lead = "Mobile capital: what holders paid for the mobile supply.")]
    pub capital: LazyFiatPerBlock<Cents>,
    /// Realized cap: the sum of each unspent output's BTC value times the spot price when it was
    /// created, weighted by its age range's mobility.
    #[traversable(wrap = "capital", rename = "realized_cap")]
    #[traversable(lead = "Mobile realized cap: what holders paid for the mobile supply, each coin at the price it last moved at.")]
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Creation-price statistics of the mobile supply: each unspent output weighted by its
    /// age range's mobility.
    pub cost_basis: CostBasisVecs<M>,
}
