use brk_error::Result;

use bitview_cohort::{
    ByTerm, CohortContext, TERM_FILTERS, TERM_NAMES, TermId, UTXO_AGGREGATE_FILTERS,
    UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Cents, Height, PartsPerMillion32, PartsPerMillionSigned32, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, Rw, StorageMode};

use crate::{AllChainSources, metrics::AggregatePercentPerBlock};
use bitview_compute::{
    CACHE_BUDGET, ColumnarPerBlock, LazyColumnPercentPerBlock, LazyPercentPerBlock, RatioCents,
    RatioDollars,
};

use super::{GrossPnlComposition, RelativeSource, SupplyProfitabilityShares};

#[derive(Traversable)]
pub struct RelativeVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub supply_profitability_shares: SupplyProfitabilityShares<M>,
    #[traversable(wrap = "unrealized/profit", rename = "to_mcap")]
    /// Unrealized profit of an aggregate UTXO cohort divided by total Bitcoin
    /// market cap at the represented block. Unrealized profit sums market value
    /// minus creation-date value only for unspent outputs whose spot price is
    /// above creation price. Returns zero when market cap is zero.
    pub unrealized_profit_to_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/loss", rename = "to_mcap")]
    /// Unrealized loss of an aggregate UTXO cohort divided by total Bitcoin
    /// market cap at the represented block. Unrealized loss sums creation-date
    /// value minus market value only for unspent outputs whose spot price is
    /// below creation price. Returns zero when market cap is zero.
    pub unrealized_loss_to_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/profit", rename = "to_own_mcap")]
    /// Unrealized profit divided by the short- or long-term-holder cohort's own
    /// market cap, its unspent BTC supply valued at the represented block's spot
    /// price. Unrealized profit sums market value minus creation-date value only
    /// for outputs above their creation price. Returns zero when cohort market
    /// cap is zero.
    pub unrealized_profit_to_own_mcap: ColumnarPerBlock<
        PartsPerMillion32,
        TermId,
        ByTerm<LazyColumnPercentPerBlock<PartsPerMillion32, TermId>>,
        M,
    >,
    #[traversable(wrap = "unrealized/loss", rename = "to_own_mcap")]
    /// Unrealized loss divided by the short- or long-term-holder cohort's own
    /// market cap, its unspent BTC supply valued at the represented block's spot
    /// price. Unrealized loss sums creation-date value minus market value only
    /// for outputs below their creation price. Returns zero when cohort market
    /// cap is zero.
    pub unrealized_loss_to_own_mcap: ColumnarPerBlock<
        PartsPerMillion32,
        TermId,
        ByTerm<LazyColumnPercentPerBlock<PartsPerMillion32, TermId>>,
        M,
    >,
    #[traversable(wrap = "unrealized/net_pnl", rename = "to_own_mcap")]
    /// Net unrealized profit and loss divided by the short- or long-term-holder
    /// cohort's own market cap. It equals `(spot price - realized price) / spot
    /// price`; positive values place spot above the cohort's aggregate on-chain
    /// cost basis and negative values place it below. A zero or unavailable
    /// market-value-to-realized-value ratio produces NaN.
    pub net_unrealized_pnl_to_own_mcap: ByTerm<LazyPercentPerBlock<PartsPerMillionSigned32>>,
    #[traversable(flatten)]
    pub gross_pnl_composition: GrossPnlComposition<M>,
    #[traversable(wrap = "invested_capital/in_profit", rename = "share")]
    /// Share of an aggregate UTXO cohort's realized cap held by unspent outputs
    /// whose creation price is less than or equal to the represented block's
    /// spot price. It divides those outputs' creation-date value by the
    /// creation-date value of all outputs in the cohort. Returns zero when
    /// realized cap is zero.
    pub invested_capital_in_profit_share: AggregatePercentPerBlock<PartsPerMillion32, M>,
    #[traversable(wrap = "invested_capital/in_loss", rename = "share")]
    /// Share of an aggregate UTXO cohort's realized cap held by unspent outputs
    /// whose creation price is greater than the represented block's spot price.
    /// It divides those outputs' creation-date value by the creation-date value
    /// of all outputs in the cohort. Returns zero when realized cap is zero.
    pub invested_capital_in_loss_share: AggregatePercentPerBlock<PartsPerMillion32, M>,
}

impl RelativeVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        all_chain: &AllChainSources,
        sources: &UTXOAggregate<RelativeSource<'_>>,
    ) -> Result<Self> {
        let aggregate_version = version + Version::ONE;
        let supply_profitability_shares =
            SupplyProfitabilityShares::forced_import(db, aggregate_version, mappings)?;
        let unrealized_profit_to_own_mcap = Self::import_term_percent(
            db,
            "unrealized_profit_to_own_mcap",
            aggregate_version,
            mappings,
        )?;
        let unrealized_loss_to_own_mcap = Self::import_term_percent(
            db,
            "unrealized_loss_to_own_mcap",
            aggregate_version,
            mappings,
        )?;
        let gross_pnl_composition =
            GrossPnlComposition::forced_import(db, aggregate_version, mappings)?;
        let invested_capital_in_profit_share = AggregatePercentPerBlock::forced_import(
            db,
            "invested_capital_in_profit_share",
            aggregate_version,
            mappings,
        )?;
        let invested_capital_in_loss_share = AggregatePercentPerBlock::forced_import(
            db,
            "invested_capital_in_loss_share",
            aggregate_version,
            mappings,
        )?;

        let unrealized_profit_to_mcap = UTXOAggregate::from_fn(|id| {
            let source = id.select(sources);
            let name = Self::aggregate_metric_name(id, "unrealized_profit_to_mcap");
            let source = all_chain.with_market_cap(
                &format!("{name}_ppm_source"),
                Version::new(2),
                &source.unrealized.profit.cents.height,
                |_, value, market_cap| Self::ratio_to_market_cap(value, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(2), source, mappings)
        });
        let unrealized_loss_to_mcap = UTXOAggregate::from_fn(|id| {
            let source = id.select(sources);
            let name = Self::aggregate_metric_name(id, "unrealized_loss_to_mcap");
            let source = all_chain.with_market_cap(
                &format!("{name}_ppm_source"),
                Version::new(2),
                &source.unrealized.loss.cents.height,
                |_, value, market_cap| Self::ratio_to_market_cap(value, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(2), source, mappings)
        });
        let net_unrealized_pnl_to_own_mcap = ByTerm::from_fn(|term_id| {
            let aggregate_id = match term_id {
                TermId::Short => UTXOAggregateId::Sth,
                TermId::Long => UTXOAggregateId::Lth,
            };
            let source = CACHE_BUDGET.wrap(aggregate_id.select(sources).nupl.ppm.height.clone());
            LazyPercentPerBlock::from_height_source(
                &Self::aggregate_metric_name(aggregate_id, "net_unrealized_pnl_to_own_mcap"),
                version + Version::new(4),
                source,
                mappings,
            )
        });

        Ok(Self {
            supply_profitability_shares,
            unrealized_profit_to_mcap,
            unrealized_loss_to_mcap,
            unrealized_profit_to_own_mcap,
            unrealized_loss_to_own_mcap,
            net_unrealized_pnl_to_own_mcap,
            gross_pnl_composition,
            invested_capital_in_profit_share,
            invested_capital_in_loss_share,
        })
    }

    fn import_term_percent(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<
        ColumnarPerBlock<
            PartsPerMillion32,
            TermId,
            ByTerm<LazyColumnPercentPerBlock<PartsPerMillion32, TermId>>,
        >,
    > {
        ColumnarPerBlock::forced_import(db, &format!("{metric}_ppm_by_term"), version, |source| {
            ByTerm::from_fn(|id| {
                let name = CohortContext::Utxo.metric_name(
                    id.select(&TERM_FILTERS),
                    id.select(&TERM_NAMES).id,
                    metric,
                );
                LazyColumnPercentPerBlock::new(&name, version, source, id, mappings)
            })
        })
    }

    fn aggregate_metric_name(id: UTXOAggregateId, metric: &str) -> String {
        CohortContext::Utxo.metric_name(
            id.select(&UTXO_AGGREGATE_FILTERS),
            id.select(&UTXO_AGGREGATE_NAMES).id,
            metric,
        )
    }

    fn ratio_to_market_cap(value: Cents, market_cap: Cents) -> PartsPerMillion32 {
        let ratio = f64::from(value) / f64::from(market_cap);
        if ratio.is_finite() {
            PartsPerMillion32::from(ratio)
        } else {
            PartsPerMillion32::default()
        }
    }

    fn term_source<'a>(
        sources: &'a UTXOAggregate<RelativeSource<'a>>,
        id: TermId,
    ) -> &'a RelativeSource<'a> {
        match id {
            TermId::Short => &sources.sth,
            TermId::Long => &sources.lth,
        }
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        sources: &UTXOAggregate<RelativeSource<'_>>,
        exit: &Exit,
    ) -> Result<()> {
        self.supply_profitability_shares
            .compute(max_from, sources, exit)?;
        self.unrealized_profit_to_own_mcap.compute_columns2(
            max_from,
            |id| &Self::term_source(sources, id).unrealized.profit.usd.height,
            |id| &Self::term_source(sources, id).supply.total.usd.height,
            |_, value, market_cap| RatioDollars::<PartsPerMillion32>::apply(value, market_cap),
            exit,
        )?;
        self.unrealized_loss_to_own_mcap.compute_columns2(
            max_from,
            |id| &Self::term_source(sources, id).unrealized.loss.usd.height,
            |id| &Self::term_source(sources, id).supply.total.usd.height,
            |_, value, market_cap| RatioDollars::<PartsPerMillion32>::apply(value, market_cap),
            exit,
        )?;
        self.gross_pnl_composition
            .compute(max_from, sources, exit)?;
        self.invested_capital_in_profit_share.compute_columns2(
            max_from,
            |id| {
                &id.select(sources)
                    .unrealized_aggregate
                    .invested_capital_in_profit
                    .cents
                    .height
            },
            |id| &id.select(sources).realized.cap.cents.height,
            |_, invested, realized_cap| {
                RatioCents::<PartsPerMillion32>::apply(invested, realized_cap)
            },
            exit,
        )?;
        self.invested_capital_in_loss_share.compute_columns2(
            max_from,
            |id| {
                &id.select(sources)
                    .unrealized_aggregate
                    .invested_capital_in_loss
                    .cents
                    .height
            },
            |id| &id.select(sources).realized.cap.cents.height,
            |_, invested, realized_cap| {
                RatioCents::<PartsPerMillion32>::apply(invested, realized_cap)
            },
            exit,
        )?;
        Ok(())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            self.supply_profitability_shares.stored_mut(),
            self.unrealized_profit_to_own_mcap.stored_mut(),
            self.unrealized_loss_to_own_mcap.stored_mut(),
            self.gross_pnl_composition.stored_mut(),
            self.invested_capital_in_profit_share.stored_mut(),
            self.invested_capital_in_loss_share.stored_mut(),
        ]
    }
}
