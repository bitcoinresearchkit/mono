use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    IndexSources, LazyPerBlock, PercentilesVecs, Price, PriceWithMvrv, PriceWithRatio,
    shapes::ByPercentile,
};

/// A cohort's creation prices: its means with their spot ratios, and the percentile views of
/// grid prices another owner stores and computes. Owners keep both as their own documented
/// `by_coin` and `by_dollar` fields.
pub struct CostBasisMeans<M: StorageMode = Rw> {
    pub by_coin: CostBasisByCoin<M>,
    pub by_dollar: CostBasisByDollar<M>,
}

#[derive(Traversable)]
pub struct CostBasisByCoin<M: StorageMode = Rw> {
    /// The weighted mean; undefined (null) while the cohort holds no supply.
    #[traversable(lead = "Realized price: what holders paid for their bitcoin, on average, each coin at the price it last moved at.")]
    pub avg: PriceWithMvrv<M>,
    /// Realized price: realized cap divided by supply, the weighted mean; undefined (null) while
    /// the cohort holds no supply.
    #[traversable(lead = "Realized price: what holders paid for their bitcoin, on average, each coin at the price it last moved at.")]
    realized_price: Price<LazyPerBlock<Cents>>,
    /// On the price grid: creation prices rounded to four significant digits of the price in cents
    /// (exact cents under $100, then steps of at most 0.1% up to $42,949,672.94, where higher
    /// prices cap). Undefined (null) without supply.
    #[traversable(flatten)]
    #[traversable(lead = "Cost basis percentile: the price that about a share of the coins last moved below, as its name says (for p25, a quarter of them).")]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Median realized price: the weighted median, on the price grid: creation prices rounded to
    /// four significant digits of the price in cents (exact cents under $100, then steps of at
    /// most 0.1% up to $42,949,672.94, where higher prices cap). Undefined (null) without supply.
    #[traversable(lead = "Median realized price: about half the coins last moved below it, half above.")]
    median_realized_price: Price<LazyPerBlock<Cents>>,
}

#[derive(Traversable)]
pub struct CostBasisByDollar<M: StorageMode = Rw> {
    /// The weighted mean; undefined (null) while the cohort's USD value at creation is zero.
    #[traversable(lead = "Capitalized price: what holders paid on average, each coin weighted by the dollars it cost.")]
    pub avg: PriceWithRatio<M>,
    /// Capitalized price: the weighted mean; undefined (null) while the cohort's USD value at
    /// creation is zero.
    #[traversable(lead = "Capitalized price: what holders paid on average, each coin weighted by the dollars it cost.")]
    capitalized_price: Price<LazyPerBlock<Cents>>,
    /// On the price grid: creation prices rounded to four significant digits of the price in cents
    /// (exact cents under $100, then steps of at most 0.1% up to $42,949,672.94, where higher
    /// prices cap). Undefined (null) while the cohort's capital is zero.
    #[traversable(flatten)]
    #[traversable(lead = "Cost basis percentile, by dollars: the price that about a share of the dollars paid was paid below, as its name says (for p25, a quarter of them).")]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Median capitalized price: the weighted median, on the price grid: creation prices rounded to
    /// four significant digits of the price in cents (exact cents under $100, then steps of at most
    /// 0.1% up to $42,949,672.94, where higher prices cap). Undefined (null) while the cohort's
    /// capital is zero.
    #[traversable(lead = "Median capitalized price: about half the dollars holders paid were paid below it.")]
    median_capitalized_price: Price<LazyPerBlock<Cents>>,
}

impl CostBasisMeans {
    /// `name` gives each id from its metric (`cost_basis_by_coin_avg` ->
    /// `sth_cost_basis_by_coin_avg`). The means are lazy over `realized_price` and
    /// `capitalized_price`, with stored spot ratios; the median aliases take `percentile_version`,
    /// the version of the percentiles' owner.
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        percentile_version: Version,
        realized_price: &impl ReadableCloneableVec<Height, Cents>,
        capitalized_price: &impl ReadableCloneableVec<Height, Cents>,
        by_coin: &PercentilesVecs,
        by_dollar: &PercentilesVecs,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            by_coin: CostBasisByCoin {
                avg: PriceWithMvrv::import(
                    db,
                    &name("cost_basis_by_coin_avg"),
                    &name("mvrv"),
                    version,
                    realized_price,
                    indexes,
                )?,
                realized_price: Price::from_height_source(
                    &name("realized_price"),
                    version,
                    realized_price,
                    indexes,
                ),
                percentiles: by_coin.prices.clone(),
                median_realized_price: Price::from_lazy_cents_source(
                    &name("median_realized_price"),
                    percentile_version,
                    &by_coin.median.cents,
                ),
            },
            by_dollar: CostBasisByDollar {
                avg: PriceWithRatio::import(
                    db,
                    &name("cost_basis_by_dollar_avg"),
                    version,
                    capitalized_price,
                    indexes,
                )?,
                capitalized_price: Price::from_height_source(
                    &name("capitalized_price"),
                    version,
                    capitalized_price,
                    indexes,
                ),
                percentiles: by_dollar.prices.clone(),
                median_capitalized_price: Price::from_lazy_cents_source(
                    &name("median_capitalized_price"),
                    percentile_version,
                    &by_dollar.median.cents,
                ),
            },
        })
    }
}
