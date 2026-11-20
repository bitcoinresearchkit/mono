use bitview_primitives::Multiple;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, Database, Ident, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    IndexSources, LazyIndexedVec, LazyPerBlock, PercentilesVecs, Price, PriceWithRatio,
    ops::Quotient, shapes::ByPercentile,
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
    #[traversable(lead = "Realized price: what the bitcoin held cost, on average, each coin at the price it last moved at.")]
    pub avg: RealizedPriceWithMvrv<M>,
    /// Realized price: realized cap divided by supply, the weighted mean; undefined (null) while
    /// the cohort holds no supply.
    #[traversable(lead = "Realized price: what the bitcoin held cost, on average, each coin at the price it last moved at.")]
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

/// `PriceWithMvrv` with its own MVRV: market cap over realized cap, not the spot ratio of the
/// cent-rounded price. It exists for that description; clients see the same `PriceWithMvrv`.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(client = "PriceWithMvrv")]
pub struct RealizedPriceWithMvrv<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    price: PriceWithRatio<M>,
    /// MVRV (market value to realized value) divides the cohort's market cap by its realized cap.
    /// It is undefined (null) while realized cap is zero.
    #[traversable(lead = "MVRV is the cohort's market cap divided by its realized cap.")]
    mvrv: LazyPerBlock<Multiple>,
}

#[derive(Traversable)]
pub struct CostBasisByDollar<M: StorageMode = Rw> {
    /// The weighted mean; undefined (null) while the cohort's USD value at creation is zero.
    #[traversable(lead = "Capitalized price: what the bitcoin held cost, on average, each coin weighted by the dollars it cost.")]
    pub avg: PriceWithRatio<M>,
    /// Capitalized price: the weighted mean; undefined (null) while the cohort's USD value at
    /// creation is zero.
    #[traversable(lead = "Capitalized price: what the bitcoin held cost, on average, each coin weighted by the dollars it cost.")]
    capitalized_price: Price<LazyPerBlock<Cents>>,
    /// On the price grid: creation prices rounded to four significant digits of the price in cents
    /// (exact cents under $100, then steps of at most 0.1% up to $42,949,672.94, where higher
    /// prices cap). Undefined (null) while the cohort's capital is zero.
    #[traversable(flatten)]
    #[traversable(lead = "Cost basis percentile, by dollars: the price that about a share of the coins, by the dollars they cost, last moved below, as its name says (for p25, a quarter of them).")]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Median capitalized price: the weighted median, on the price grid: creation prices rounded to
    /// four significant digits of the price in cents (exact cents under $100, then steps of at most
    /// 0.1% up to $42,949,672.94, where higher prices cap). Undefined (null) while the cohort's
    /// capital is zero.
    #[traversable(lead = "Median capitalized price: about half the coins, by the dollars they cost, last moved below it.")]
    median_capitalized_price: Price<LazyPerBlock<Cents>>,
}

impl CostBasisMeans {
    /// `name` gives each id from its metric (`cost_basis_by_coin_avg` ->
    /// `sth_cost_basis_by_coin_avg`). The means are lazy over `realized_price` and
    /// `capitalized_price`, with stored spot ratios, and the MVRV is lazy over `market_cap` and
    /// `realized_cap`; the median aliases take `percentile_version`, the version of the
    /// percentiles' owner.
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        percentile_version: Version,
        realized_price: &impl ReadableCloneableVec<Height, Cents>,
        capitalized_price: &impl ReadableCloneableVec<Height, Cents>,
        market_cap: &impl ReadableCloneableVec<Height, Dollars>,
        realized_cap: &impl ReadableCloneableVec<Height, Cents>,
        by_coin: &PercentilesVecs,
        by_dollar: &PercentilesVecs,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let mvrv_name = name("mvrv");
        let mvrv_source = LazyIndexedVec::new(
            &format!("{mvrv_name}_source"),
            version,
            market_cap,
            realized_cap,
            |_, market: Dollars, realized: Cents| {
                Quotient::<Multiple>::apply(market, Dollars::from(realized))
            },
        );
        Ok(Self {
            by_coin: CostBasisByCoin {
                avg: RealizedPriceWithMvrv {
                    price: PriceWithRatio::import(
                        db,
                        &name("cost_basis_by_coin_avg"),
                        version,
                        realized_price,
                        indexes,
                    )?,
                    mvrv: LazyPerBlock::from_height_source::<Ident>(
                        &mvrv_name,
                        version,
                        &mvrv_source,
                        indexes,
                    ),
                },
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
