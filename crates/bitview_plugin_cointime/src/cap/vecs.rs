use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32};
use vecdb::{Rw, StorageMode};

use bitview_compute::{FiatPerBlock, LazyFiatPerBlock, RatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Cumulative USD value, at each block's spot price, of the derived subsidy
    /// component equal to coinbase output value minus transaction fees.
    pub thermo: LazyFiatPerBlock<Cents>,
    /// Realized capitalization minus thermo capitalization.
    pub investor: FiatPerBlock<Cents, M>,
    /// Realized capitalization multiplied by vaultedness.
    pub vaulted: FiatPerBlock<Cents, M>,
    /// Realized capitalization multiplied by liveliness.
    pub active: FiatPerBlock<Cents, M>,
    /// Cumulative cointime value destroyed multiplied by circulating supply,
    /// then divided by cumulative coinblocks stored.
    pub cointime: FiatPerBlock<Cents, M>,
    /// Active capitalization divided by investor capitalization.
    pub aviv: RatioPerBlock<PartsPerMillion32, M>,
}
