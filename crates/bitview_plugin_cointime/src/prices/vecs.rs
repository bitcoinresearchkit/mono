use bitview_primitives::{PartsPerMillionSigned32, PriceRatio, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, LazyRatioPerBlock, PriceWithMvrv, PriceWithRatio};
use brk_types::{Cents, Height};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Realized price divided by one minus liveliness, where liveliness is
    /// cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created. This raises realized price as a larger share of accumulated
    /// holding time is consumed rather than stored.
    pub vaulted: PriceWithMvrv<M>,
    /// Realized price divided by liveliness, where liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created.
    /// This raises realized price when little accumulated holding time has been
    /// consumed.
    pub active: PriceWithMvrv<M>,
    /// Investor capitalization, equal to realized capitalization minus the
    /// cumulative issuance-date USD value of the derived block-subsidy
    /// component, divided by active supply in BTC. Active supply is circulating
    /// supply multiplied by liveliness.
    pub true_market_mean: PriceWithRatio<M>,
    /// AVIV: active capitalization divided by investor capitalization, which
    /// equals spot price divided by True Market Mean (the same series as
    /// `true_market_mean.ratio`).
    #[traversable(wrap = "true_market_mean", rename = "aviv")]
    pub(super) aviv: LazyPerBlock<Ratio, Ratio>,
    /// AVIV NUPL: one minus one over AVIV, the share of active capitalization
    /// above investor capitalization.
    #[traversable(wrap = "true_market_mean", rename = "aviv_nupl")]
    pub(super) aviv_nupl: LazyRatioPerBlock<PartsPerMillionSigned32, PriceRatio>,
    /// Cumulative cointime value destroyed divided by cumulative coinblocks
    /// stored, expressed as a price per BTC. It represents the average value
    /// destroyed for each unit of holding time that remains stored.
    pub cointime: PriceWithMvrv<M>,
    #[traversable(hidden)]
    pub(super) vaulted_cents: CachedSeries<Height, Cents, M>,
    #[traversable(hidden)]
    pub(super) active_cents: CachedSeries<Height, Cents, M>,
    #[traversable(hidden)]
    pub(super) true_market_mean_cents: CachedSeries<Height, Cents, M>,
}
