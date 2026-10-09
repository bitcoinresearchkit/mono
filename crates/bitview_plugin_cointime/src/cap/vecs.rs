use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{FiatPerBlock, LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock};
use brk_types::{Cents, Dollars};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Thermo capitalization: cumulative USD value, at each issuance block's
    /// spot price, of the derived subsidy component equal to coinbase output
    /// value minus transaction fees. It estimates the historical value assigned
    /// to miners through issuance, rather than valuing subsidies at current spot.
    pub thermo: LazyFiatPerBlock<Cents>,
    /// Investor capitalization: realized capitalization minus thermo
    /// capitalization. It estimates the creation-date capital attributed to
    /// market investors after removing the issuance-date value assigned to
    /// miners.
    pub investor: FiatPerBlock<Cents, M>,
    /// Active capitalization: market capitalization multiplied by liveliness,
    /// the value of the active supply.
    pub active: LazyPerBlock<Dollars>,
    /// Vaulted capitalization: market capitalization multiplied by vaultedness,
    /// the value of the vaulted supply.
    pub vaulted: LazyPerBlock<Dollars>,
    /// Cointime capitalization: cumulative sum of spot price times coinblocks
    /// destroyed, divided by cumulative coinblocks stored, then multiplied by
    /// circulating supply. It values the supply using the average destroyed
    /// value per unit of holding time that remains stored.
    pub cointime: FiatPerBlock<Cents, M>,
    /// Investor capitalization's share of realized capitalization. Zero while
    /// realized capitalization is zero.
    pub investorness: LazyPercentPerBlock<PartsPerMillion32>,
    /// Thermo capitalization's share of realized capitalization. Zero while
    /// realized capitalization is zero.
    pub producerness: LazyPercentPerBlock<PartsPerMillion32>,
}
