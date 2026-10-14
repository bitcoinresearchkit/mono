use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, Ident, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    IndexSources, LazyPerBlock, PercentilesVecs, Price, PriceWithMvrv, PriceWithRatio,
    shapes::ByPercentile,
};

/// A cohort's creation prices: its means with their spot ratios, and the percentile views of
/// grid prices another owner stores and computes. Owners keep both as their own documented
/// `per_coin` and `per_dollar` fields.
pub struct CostBasisMeans<M: StorageMode = Rw> {
    pub per_coin: CostBasisPerCoin<M>,
    pub per_dollar: CostBasisPerDollar<M>,
}

#[derive(Traversable)]
pub struct CostBasisPerCoin<M: StorageMode = Rw> {
    /// The weighted mean; undefined (null) while the cohort holds no supply.
    pub avg: PriceWithMvrv<M>,
    /// Realized price: realized cap divided by supply, the weighted mean; undefined (null) while
    /// the cohort holds no supply.
    realized_price: Price<LazyPerBlock<Cents>>,
    /// On the price grid: creation prices rounded to four significant digits of the price in
    /// cents (exact cents under $100, then steps of at most 0.1% up to $100,000,000). Undefined
    /// (null) without supply.
    #[traversable(flatten)]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Median realized price: the weighted median, on the price grid: creation prices rounded to
    /// four significant digits of the price in cents (exact cents under $100, then steps of at
    /// most 0.1% up to $100,000,000). Undefined (null) without supply.
    median_realized_price: Price<LazyPerBlock<Cents>>,
}

#[derive(Traversable)]
pub struct CostBasisPerDollar<M: StorageMode = Rw> {
    /// The weighted mean; undefined (null) while the cohort's USD value at creation is zero.
    pub avg: PriceWithRatio<M>,
    /// Capitalized price: the weighted mean; undefined (null) while the cohort's USD value at
    /// creation is zero.
    capitalized_price: Price<LazyPerBlock<Cents>>,
    /// On the price grid: creation prices rounded to four significant digits of the price in
    /// cents (exact cents under $100, then steps of at most 0.1% up to $100,000,000). Undefined
    /// (null) while the cohort's capital is zero.
    #[traversable(flatten)]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
}

impl CostBasisMeans {
    /// `name` gives each id from its metric (`cost_basis_per_coin_avg` ->
    /// `sth_cost_basis_per_coin_avg`). The means are lazy over `realized_price` and
    /// `capitalized_price`, with stored spot ratios; the median alias takes `percentile_version`,
    /// the version of the percentiles' owner.
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        percentile_version: Version,
        realized_price: &impl ReadableCloneableVec<Height, Cents>,
        capitalized_price: &impl ReadableCloneableVec<Height, Cents>,
        per_coin: &PercentilesVecs,
        per_dollar: &PercentilesVecs,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            per_coin: CostBasisPerCoin {
                avg: PriceWithMvrv::import(
                    db,
                    &name("cost_basis_per_coin_avg"),
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
                percentiles: per_coin.prices.clone(),
                median_realized_price: Price::from_lazy_cents_source::<Ident, Cents>(
                    &name("median_realized_price"),
                    percentile_version,
                    &per_coin.median.cents,
                ),
            },
            per_dollar: CostBasisPerDollar {
                avg: PriceWithRatio::import(
                    db,
                    &name("cost_basis_per_dollar_avg"),
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
                percentiles: per_dollar.prices.clone(),
            },
        })
    }
}
