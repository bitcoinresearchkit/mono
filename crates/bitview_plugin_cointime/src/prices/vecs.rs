use bitview_primitives::{Float64, Multiple, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazyRatioPerBlock, Price, PriceWithMvrv, PriceWithRatio,
};
use brk_types::{Cents, Dollars, Height};
use vecdb::{PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Realized price divided by one minus liveliness, where liveliness is
    /// cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created. This raises realized price as a larger share of accumulated
    /// holding time is consumed rather than stored.
    #[traversable(client = "CointimePriceWithMvrv")]
    #[traversable(lead = "Vaulted price: the realized price, scaled up by how much of the supply sits unmoved.")]
    pub vaulted: PriceWithMvrv<M>,
    /// Realized price divided by liveliness, where liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created.
    /// This raises realized price when little accumulated holding time has been
    /// consumed.
    #[traversable(client = "CointimePriceWithMvrv")]
    #[traversable(lead = "Active price: the realized price, scaled by how much of the supply is in use.")]
    pub active: PriceWithMvrv<M>,
    /// Investor capitalization, equal to realized capitalization minus the
    /// cumulative issuance-date USD value of the derived block-subsidy
    /// component, divided by active supply in BTC. Active supply is circulating
    /// supply multiplied by liveliness. Undefined (null) unless both investor
    /// capitalization and active supply are positive.
    #[traversable(client = "CointimePriceWithRatio")]
    #[traversable(lead = "True market mean: the realized cap without what miners received, per coin of the supply in use.")]
    pub true_market_mean: PriceWithRatio<M>,
    /// AVIV: active capitalization divided by investor capitalization, which
    /// equals spot price divided by True Market Mean (the same series as
    /// `true_market_mean_ratio`). Undefined (null) while True Market Mean is
    /// null or rounds to zero cents.
    #[traversable(wrap = "true_market_mean", rename = "aviv")]
    #[traversable(lead = "AVIV: the price against the true market mean.")]
    pub(super) aviv: LazyPerBlock<Multiple>,
    /// AVIV NUPL: active capitalization minus investor capitalization, divided by
    /// active capitalization (one minus one over AVIV where AVIV is defined). It
    /// exceeds one while investor capitalization is negative, including periods
    /// when AVIV is null. Undefined (null) while active capitalization is zero.
    #[traversable(wrap = "true_market_mean", rename = "aviv_nupl")]
    #[traversable(lead = "AVIV NUPL: the active cap above the investor cap, as a share of the active cap.")]
    pub(super) aviv_nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
    /// Cumulative cointime value destroyed divided by cumulative coinblocks
    /// stored, expressed as a price per BTC. It represents the average value
    /// destroyed for each unit of holding time that remains stored.
    #[traversable(client = "CointimePriceWithMvrv")]
    #[traversable(lead = "Cointime price: the value of the holding time spent, for each unit stored.")]
    pub cointime: PriceWithMvrv<M>,
    /// Transferred price (David Puell, in Willy Woo's "Experiments on Cumulative Destruction",
    /// 2019): `sum(coin days destroyed × price) / (days × supply)`. Each block's coin days
    /// destroyed are valued at its spot price in USD, summed from genesis, and divided by the
    /// market's age in days and by circulating supply in BTC. The source names no start for the
    /// market's age: it is counted from the genesis block's day (2009-01-03), the start that
    /// reproduces the published CVDD.
    #[traversable(lead = "Transferred price: the value of all coin days destroyed, per coin and per day of the market's age.")]
    pub transferred: Price<LazyPerBlock<Cents>>,
    /// Balanced price (David Puell, same article): `realized price − transferred price`, here as
    /// realized capitalization less the transferred price's capitalization, divided by
    /// circulating supply in BTC. Zero where the difference is negative.
    #[traversable(lead = "Balanced price: the realized price less the transferred price.")]
    pub balanced: Price<LazyPerBlock<Cents>>,
    /// Terminal price (Checkmate): the transferred price times 21, as read in David Puell's thread
    /// of 2021-11-05 through a Thread Reader copy (read 2026-11-26,
    /// https://threadreaderapp.com/thread/1456748037364240396.html): "terminal price = transferred
    /// price * 21". A model of tops: it runs far above the price at the lows.
    #[traversable(lead = "Terminal price: the transferred price times 21.")]
    pub terminal: Price<LazyPerBlock<Cents>>,
    /// CVDD, cumulative value-days destroyed (Willy Woo, "Experiments on Cumulative
    /// Destruction", 2019): `sum(coin days destroyed × price) / (days × 6000000)`, the 6 million
    /// being the source's own calibration. Each block's coin days destroyed are valued at its
    /// spot price in USD and summed from genesis; the market's age is in days from the genesis
    /// block's day (2009-01-03).
    #[traversable(lead = "CVDD: the value of all coin days destroyed, over the market's age in days and 6 million.")]
    pub cvdd: Price<LazyPerBlock<Cents>>,
    /// Delta price: David Puell's Delta Cap (realized capitalization minus average
    /// capitalization) divided by circulating supply in BTC. Average capitalization is market
    /// capitalization's life-to-date mean from the first exchange price (2010), over time: each
    /// block's cap held until the next. A second per-coin wording in circulation (realized price
    /// minus the life-to-date mean price) runs a few percent lower. Zero where the difference is
    /// negative.
    #[traversable(lead = "Delta price: the realized cap less the average cap, per coin.")]
    pub delta: Price<LazyPerBlock<Cents>>,
    /// Running sum of coin days destroyed times spot price in USD, in USD-days.
    #[traversable(hidden)]
    pub(crate) value_days_destroyed_sum: M::Stored<PcoVec<Height, Float64>>,
    /// Average capitalization in USD: market capitalization's mean over time since the first price,
    /// each block's cap held until the next block; zero before it, as market capitalization is.
    #[traversable(hidden)]
    pub(crate) average_cap: M::Stored<PcoVec<Height, Dollars>>,
    #[traversable(hidden)]
    pub(super) vaulted_cents: CachedSeries<Cents, M>,
    #[traversable(hidden)]
    pub(super) active_cents: CachedSeries<Cents, M>,
    #[traversable(hidden)]
    pub(super) true_market_mean_cents: CachedSeries<Cents, M>,
}
