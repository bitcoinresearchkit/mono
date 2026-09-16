use bitview_traversable::Traversable;
use bitview_vecs::LazyPriceWithRatioPerBlock;
use vecdb::{Rw, StorageMode};

use super::{CohortVecs, Sources};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Uses all UTXO age ranges.
    pub all: CohortVecs,
    /// Uses short-term-holder UTXO age ranges younger than 150 days.
    pub sth: CohortVecs,
    /// Uses long-term-holder UTXO age ranges at least 150 days old.
    pub lth: CohortVecs,
    /// Awake-weighted realized price below 4 months.
    pub under_4m_awake_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted capitalized price below 4 months.
    pub under_4m_awake_capitalized_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted realized price below 6 months.
    pub under_6m_awake_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted capitalized price below 6 months.
    pub under_6m_awake_capitalized_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted realized price of UTXOs at least 120 days old.
    pub over_4m_awake_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted capitalized price of UTXOs at least 120 days old.
    pub over_4m_awake_capitalized_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted realized price of UTXOs at least 180 days old.
    pub over_6m_awake_price: LazyPriceWithRatioPerBlock,
    /// Awake-weighted capitalized price of UTXOs at least 180 days old.
    pub over_6m_awake_capitalized_price: LazyPriceWithRatioPerBlock,
    /// Height-indexed source stored for all, short-term-holder, and
    /// long-term-holder cointime aggregates.
    #[traversable(hidden)]
    pub sources: Sources<M>,
}
