use bitview_cohort::{ByTerm, CohortContext, CohortId, Term, UTXOAggregate, UTXOAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{RatioCents, RatioDollars};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyIndexedVec, LazyPercentPerBlock, PercentPerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, PartsPerMillion32, PartsPerMillionSigned32, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode};

use super::{GrossPnlComposition, RelativeSource, SupplyProfitabilityShares};
use crate::metrics::AggregatePercentPerBlock;

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
    pub unrealized_profit_to_own_mcap: ByTerm<PercentPerBlock<PartsPerMillion32, M>>,
    #[traversable(wrap = "unrealized/loss", rename = "to_own_mcap")]
    /// Unrealized loss divided by the short- or long-term-holder cohort's own
    /// market cap, its unspent BTC supply valued at the represented block's spot
    /// price. Unrealized loss sums creation-date value minus market value only
    /// for outputs below their creation price. Returns zero when cohort market
    /// cap is zero.
    pub unrealized_loss_to_own_mcap: ByTerm<PercentPerBlock<PartsPerMillion32, M>>,
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
        mappings: &MappingsVecs,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
        sources: &UTXOAggregate<RelativeSource<'_>>,
    ) -> Result<Box<Self>> {
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
            let name = id.metric_name("unrealized_profit_to_mcap");
            let source = LazyIndexedVec::new(
                &format!("{name}_ppm_source"),
                Version::new(2),
                &source.unrealized.profit.cents.height,
                all_market_cap,
                |_, value, market_cap| Self::ratio_to_market_cap(value, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(2), &source, mappings)
        });
        let unrealized_loss_to_mcap = UTXOAggregate::from_fn(|id| {
            let source = id.select(sources);
            let name = id.metric_name("unrealized_loss_to_mcap");
            let source = LazyIndexedVec::new(
                &format!("{name}_ppm_source"),
                Version::new(2),
                &source.unrealized.loss.cents.height,
                all_market_cap,
                |_, value, market_cap| Self::ratio_to_market_cap(value, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(2), &source, mappings)
        });
        let net_unrealized_pnl_to_own_mcap = ByTerm::from_fn(|term_id| {
            let aggregate_id = match term_id {
                Term::Sth => UTXOAggregateId::Sth,
                Term::Lth => UTXOAggregateId::Lth,
            };
            let source = aggregate_id.select(sources).nupl.ppm.height.clone();
            LazyPercentPerBlock::from_height_source(
                &aggregate_id.metric_name("net_unrealized_pnl_to_own_mcap"),
                version + Version::new(4),
                &source,
                mappings,
            )
        });

        Ok(Box::new(Self {
            supply_profitability_shares,
            unrealized_profit_to_mcap,
            unrealized_loss_to_mcap,
            unrealized_profit_to_own_mcap,
            unrealized_loss_to_own_mcap,
            net_unrealized_pnl_to_own_mcap,
            gross_pnl_composition,
            invested_capital_in_profit_share,
            invested_capital_in_loss_share,
        }))
    }

    fn import_term_percent(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<ByTerm<PercentPerBlock<PartsPerMillion32>>> {
        ByTerm::try_from_fn(|id| {
            let name = CohortContext::Utxo.metric_name(CohortId::Term(id), metric);
            PercentPerBlock::forced_import(db, &name, version + Version::ONE, mappings)
        })
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
        id: Term,
    ) -> &'a RelativeSource<'a> {
        match id {
            Term::Sth => &sources.sth,
            Term::Lth => &sources.lth,
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
        for id in [Term::Sth, Term::Lth] {
            let source = Self::term_source(sources, id);
            self.unrealized_profit_to_own_mcap
                .get_mut(id)
                .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                    max_from,
                    &source.unrealized.profit.usd.height,
                    &source.supply.total.usd.height,
                    exit,
                )?;
            self.unrealized_loss_to_own_mcap
                .get_mut(id)
                .compute_binary::<_, _, RatioDollars<PartsPerMillion32>>(
                    max_from,
                    &source.unrealized.loss.usd.height,
                    &source.supply.total.usd.height,
                    exit,
                )?;
        }
        self.gross_pnl_composition
            .compute(max_from, sources, exit)?;
        for id in UTXOAggregateId::ALL {
            let source = id.select(sources);
            for (target, invested) in [
                (
                    id.select_mut(&mut self.invested_capital_in_profit_share.stored),
                    &source
                        .unrealized_aggregate
                        .invested_capital_in_profit
                        .cents
                        .height,
                ),
                (
                    id.select_mut(&mut self.invested_capital_in_loss_share.stored),
                    &source
                        .unrealized_aggregate
                        .invested_capital_in_loss
                        .cents
                        .height,
                ),
            ] {
                target.compute_transform2(
                    max_from,
                    invested,
                    &source.realized.cap.cents.height,
                    |(height, invested, realized_cap, _)| {
                        (
                            height,
                            RatioCents::<PartsPerMillion32>::apply(invested, realized_cap),
                        )
                    },
                    exit,
                )?;
            }
        }
        Ok(())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.supply_profitability_shares.collect_vecs_mut();
        vecs.extend(self.gross_pnl_composition.collect_vecs_mut());
        vecs.extend(
            self.unrealized_profit_to_own_mcap
                .iter_mut()
                .chain(self.unrealized_loss_to_own_mcap.iter_mut())
                .map(|v| &mut v.ppm.height as &mut dyn AnyStoredVec),
        );
        vecs.extend(self.invested_capital_in_profit_share.collect_vecs_mut());
        vecs.extend(self.invested_capital_in_loss_share.collect_vecs_mut());
        vecs
    }
}
