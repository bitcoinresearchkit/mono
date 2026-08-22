use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32};
use vecdb::{Rw, StorageMode};

use bitview_compute::{FiatPerBlock, LazyFiatPerBlock, RatioPerBlock};

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
    /// Vaulted capitalization: realized capitalization multiplied by one minus
    /// liveliness, where liveliness is cumulative coinblocks destroyed divided
    /// by cumulative coinblocks created. It attributes more creation-date
    /// capital to holding time that remains stored rather than consumed.
    pub vaulted: FiatPerBlock<Cents, M>,
    /// Active capitalization: realized capitalization multiplied by
    /// liveliness, the ratio of cumulative coinblocks destroyed to cumulative
    /// coinblocks created. It attributes creation-date capital to holding time
    /// that has been consumed by spending.
    pub active: FiatPerBlock<Cents, M>,
    /// Cointime capitalization: cumulative sum of spot price times coinblocks
    /// destroyed, divided by cumulative coinblocks stored, then multiplied by
    /// circulating supply. It values the supply using the average destroyed
    /// value per unit of holding time that remains stored.
    pub cointime: FiatPerBlock<Cents, M>,
    /// Active-value-to-investor-value (AVIV) ratio: active capitalization
    /// divided by investor capitalization. Values above one mean the
    /// liveliness-weighted realized capitalization exceeds the capital value
    /// attributed to investors after removing issuance-date subsidy value;
    /// values below one mean it is smaller.
    pub aviv: RatioPerBlock<PartsPerMillion32, M>,
}
