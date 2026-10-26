use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{FiatPerBlock, LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock};
use brk_types::{Cents, CentsSigned, Dollars};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Thermo capitalization: cumulative USD value, at each issuance block's
    /// spot price, of the derived subsidy component equal to coinbase output
    /// value minus transaction fees, floored at zero. It estimates the historical value assigned
    /// to miners through issuance, rather than valuing subsidies at current spot.
    #[traversable(lead = "Thermo cap: what miners were paid in new coins, each at the price of its day.")]
    pub thermo: LazyFiatPerBlock<Cents>,
    /// Investor capitalization: realized capitalization minus thermo
    /// capitalization. It estimates the creation-date capital attributed to
    /// market investors after removing the issuance-date value assigned to
    /// miners, and turns negative when coins were re-acquired for less than
    /// the value issued.
    #[traversable(lead = "Investor cap: the realized cap without what miners received: what investors paid.")]
    pub investor: FiatPerBlock<CentsSigned, M>,
    /// Active capitalization: market capitalization multiplied by liveliness,
    /// the value of the active supply. Reported in US dollars.
    #[traversable(lead = "Active cap: the market value of the supply in use.")]
    pub active: LazyPerBlock<Dollars>,
    /// Vaulted capitalization: market capitalization multiplied by vaultedness,
    /// the value of the vaulted supply. Reported in US dollars.
    #[traversable(lead = "Vaulted cap: the market value of the supply sitting unmoved.")]
    pub vaulted: LazyPerBlock<Dollars>,
    /// Cointime capitalization: cumulative sum of spot price times coinblocks
    /// destroyed, divided by cumulative coinblocks stored, then multiplied by
    /// circulating supply. It values the supply using the average destroyed
    /// value per unit of holding time that remains stored.
    #[traversable(lead = "Cointime cap: the value of the holding time spent, against the time stored, for the whole supply.")]
    pub cointime: FiatPerBlock<Cents, M>,
    /// Investor capitalization's share of realized capitalization, negative while investor
    /// capitalization is. Undefined (null) while realized capitalization is zero.
    #[traversable(lead = "Investors' share of the realized cap.")]
    pub investorness: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Thermo capitalization's share of realized capitalization, above 100% while investor
    /// capitalization is negative. Undefined (null) while realized capitalization is zero.
    #[traversable(lead = "Miners' share of the realized cap.")]
    pub producerness: LazyPercentPerBlock<PartsPerMillion32>,
}
