use bitview_primitives::{Multiple, PartsPerMillionSigned32};
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
    #[traversable(lead = "Vaulted price: what holders paid on average, scaled up by how much of the supply sits unmoved.")]
    pub vaulted: PriceWithMvrv<M>,
    /// Realized price divided by liveliness, where liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created.
    /// This raises realized price when little accumulated holding time has been
    /// consumed.
    #[traversable(lead = "Active price: what holders paid on average, scaled by how much of the supply is in use.")]
    pub active: PriceWithMvrv<M>,
    /// Investor capitalization, equal to realized capitalization minus the
    /// cumulative issuance-date USD value of the derived block-subsidy
    /// component, divided by active supply in BTC. Active supply is circulating
    /// supply multiplied by liveliness. Undefined (null) unless both investor
    /// capitalization and active supply are positive.
    #[traversable(lead = "True market mean: what active investors paid on average, leaving out what miners received.")]
    pub true_market_mean: PriceWithRatio<M>,
    /// AVIV: active capitalization divided by investor capitalization, which
    /// equals spot price divided by True Market Mean (the same series as
    /// `true_market_mean_ratio`). Undefined (null) while True Market Mean is
    /// null or rounds to zero cents.
    #[traversable(wrap = "true_market_mean", rename = "aviv")]
    #[traversable(lead = "AVIV: the price against the true market mean. Above 1, active investors are in profit on average.")]
    pub(super) aviv: LazyPerBlock<Multiple>,
    /// AVIV NUPL: active capitalization minus investor capitalization, divided by
    /// active capitalization (one minus one over AVIV where AVIV is defined). It
    /// exceeds one while investor capitalization is negative, including periods
    /// when AVIV is null. Undefined (null) while active capitalization is zero.
    #[traversable(wrap = "true_market_mean", rename = "aviv_nupl")]
    #[traversable(lead = "AVIV NUPL: active investors' paper gains, as a share of the active supply's market value.")]
    pub(super) aviv_nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
    /// Cumulative cointime value destroyed divided by cumulative coinblocks
    /// stored, expressed as a price per BTC. It represents the average value
    /// destroyed for each unit of holding time that remains stored.
    #[traversable(lead = "Cointime price: the value of the holding time spent, for each unit stored.")]
    pub cointime: PriceWithMvrv<M>,
    #[traversable(hidden)]
    pub(super) vaulted_cents: CachedSeries<Height, Cents, M>,
    #[traversable(hidden)]
    pub(super) active_cents: CachedSeries<Height, Cents, M>,
    #[traversable(hidden)]
    pub(super) true_market_mean_cents: CachedSeries<Height, Cents, M>,
}
