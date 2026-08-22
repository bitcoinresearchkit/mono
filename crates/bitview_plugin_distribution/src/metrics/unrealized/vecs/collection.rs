use brk_error::Result;

use bitview_cohort::{CohortContext, Filter, UTXOAggregate, UTXOGroups, UTXOGroupsWithoutAmount};
use bitview_traversable::Traversable;
use brk_types::{
    Cents, CentsSigned, CentsSquaredSats, Dollars, PartsPerMillion64, PartsPerMillionSigned32,
    Sats, Version,
};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    metrics::{AdditiveAggregateFiatPerBlock, AdditiveUTXORawVec, AggregateFiatPerBlock, UTXORows},
    state::UnrealizedState,
};
use bitview_compute::{
    LazyPerBlock, LazyPriceWithRatioPerBlock, LazyRatioPerBlock, NegCentsUnsignedToDollars,
};

use super::{
    super::{MvrvToNupl, UnrealizedAggregateSources},
    NetUnrealizedByCohort, UnrealizedByCohort, UnrealizedSources,
};

#[derive(Traversable)]
pub struct UnrealizedVecs<M: StorageMode = Rw> {
    /// Unrealized profit of a UTXO cohort's unspent outputs: market value at
    /// the represented block minus creation-date value, summed where spot is
    /// above creation price.
    pub profit: UnrealizedByCohort<Cents, M>,
    /// Unrealized loss of a UTXO cohort's unspent outputs: creation-date value
    /// minus market value at the represented block, summed where spot is below
    /// creation price.
    pub loss: UnrealizedByCohort<Cents, M>,
    /// Net unrealized profit and loss of a UTXO cohort: unrealized profit
    /// minus unrealized loss.
    pub net_pnl: NetUnrealizedByCohort<M>,
    /// Gross unrealized profit and loss of an aggregate UTXO cohort:
    /// unrealized profit plus unrealized loss.
    pub gross_pnl: AdditiveAggregateFiatPerBlock<Cents, M>,
    /// Creation-date value of an aggregate UTXO cohort's unspent outputs whose
    /// creation price is less than or equal to the represented block's spot
    /// price.
    pub invested_capital_in_profit: AdditiveAggregateFiatPerBlock<Cents, M>,
    /// Creation-date value of an aggregate UTXO cohort's unspent outputs whose
    /// creation price is greater than the represented block's spot price.
    pub invested_capital_in_loss: AdditiveAggregateFiatPerBlock<Cents, M>,
    /// Sum of creation price squared times unspent sats for a UTXO cohort's
    /// outputs whose creation price is less than or equal to the represented
    /// block's spot price. This raw numerator underlies the profit-side
    /// capitalized price.
    pub capitalized_cap_in_profit_raw: AdditiveUTXORawVec<CentsSquaredSats, M>,
    /// Sum of creation price squared times unspent sats for a UTXO cohort's
    /// outputs whose creation price is greater than the represented block's
    /// spot price. This raw numerator underlies the loss-side capitalized price.
    pub capitalized_cap_in_loss_raw: AdditiveUTXORawVec<CentsSquaredSats, M>,
    /// Pain index of an aggregate UTXO cohort: the capital-weighted creation
    /// price of its unspent supply in loss minus the represented block's spot
    /// price. Larger values mean the loss-side capital is further underwater;
    /// returns zero when the cohort has no loss-side invested capital.
    pub pain_index: AggregateFiatPerBlock<Cents, M>,
    /// Greed index of an aggregate UTXO cohort: the represented block's spot
    /// price minus the capital-weighted creation price of its unspent supply in
    /// profit. Larger values mean the profit-side capital is further above its
    /// cost basis. When the cohort has no profit-side invested capital, its
    /// capital-weighted creation price is zero and this index equals spot.
    pub greed_index: AggregateFiatPerBlock<Cents, M>,
    /// Net sentiment of an aggregate UTXO cohort: greed index minus pain index.
    /// Positive values mean the profit-side distance above cost basis exceeds
    /// the loss-side distance below it; negative values mean the reverse.
    pub net_sentiment: AggregateFiatPerBlock<CentsSigned, M>,
    /// Net unrealized profit/loss (NUPL) of a UTXO cohort as a share of that
    /// cohort's own market cap, derived from market-value-to-realized-value
    /// ratio (MVRV) as `1 - 1 / MVRV`. Positive values mean aggregate
    /// unrealized profit, negative values mean aggregate unrealized loss, and
    /// zero means spot equals the cohort's realized price. A zero or unavailable
    /// MVRV produces NaN.
    pub nupl: UTXOGroups<LazyRatioPerBlock<PartsPerMillionSigned32, PartsPerMillion64>>,
    #[traversable(wrap = "loss", rename = "negative")]
    /// Unrealized loss of a UTXO cohort's unspent outputs, expressed as a
    /// negative value.
    pub negative_loss: UTXOGroupsWithoutAmount<LazyPerBlock<Dollars, Cents>>,
}

impl UnrealizedVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        realized_price: &UTXOGroups<LazyPriceWithRatioPerBlock>,
    ) -> Result<Self> {
        let profit = UnrealizedByCohort::forced_import(
            db,
            "unrealized_profit",
            version + Version::ONE,
            mappings,
        )?;
        let loss = UnrealizedByCohort::forced_import(
            db,
            "unrealized_loss",
            version + Version::ONE,
            mappings,
        )?;
        let net_pnl = NetUnrealizedByCohort::forced_import(db, version, mappings)?;
        let aggregate_version = version + Version::ONE;
        let gross_pnl = AdditiveAggregateFiatPerBlock::forced_import(
            db,
            "unrealized_gross_pnl",
            aggregate_version,
            mappings,
        )?;
        let invested_capital_in_profit = AdditiveAggregateFiatPerBlock::forced_import(
            db,
            "invested_capital_in_profit",
            aggregate_version,
            mappings,
        )?;
        let invested_capital_in_loss = AdditiveAggregateFiatPerBlock::forced_import(
            db,
            "invested_capital_in_loss",
            aggregate_version,
            mappings,
        )?;
        let capitalized_cap_in_profit_raw =
            AdditiveUTXORawVec::forced_import(db, "capitalized_cap_in_profit_raw", version)?;
        let capitalized_cap_in_loss_raw =
            AdditiveUTXORawVec::forced_import(db, "capitalized_cap_in_loss_raw", version)?;
        let pain_index =
            AggregateFiatPerBlock::forced_import(db, "pain_index", aggregate_version, mappings)?;
        let greed_index =
            AggregateFiatPerBlock::forced_import(db, "greed_index", aggregate_version, mappings)?;
        let net_sentiment =
            AggregateFiatPerBlock::forced_import(db, "net_sentiment", aggregate_version, mappings)?;
        let nupl = realized_price.map_named(|filter, cohort_name, price| {
            LazyRatioPerBlock::from_lazy_source::<MvrvToNupl, PartsPerMillion64>(
                &CohortContext::Utxo.metric_name(filter, cohort_name, "nupl"),
                Self::cohort_version(version, filter) + Version::new(5),
                &price.ppm,
            )
        });
        let negative_loss = UTXOGroupsWithoutAmount::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "unrealized_loss_neg");
            LazyPerBlock::from_lazy::<NegCentsUnsignedToDollars, Cents>(
                &name,
                Self::cohort_version(version, &filter),
                &loss
                    .cohorts
                    .get(&filter)
                    .expect("unrealized-loss cohort")
                    .cents,
            )
        });
        Ok(Self {
            profit,
            loss,
            net_pnl,
            gross_pnl,
            invested_capital_in_profit,
            invested_capital_in_loss,
            capitalized_cap_in_profit_raw,
            capitalized_cap_in_loss_raw,
            pain_index,
            greed_index,
            net_sentiment,
            nupl,
            negative_loss,
        })
    }

    fn cohort_version(version: Version, filter: &Filter) -> Version {
        version
            + if matches!(filter, Filter::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    pub fn sources(&self, filter: &Filter) -> Option<UnrealizedSources> {
        Some(UnrealizedSources {
            profit: self.profit.cohorts.get(filter)?.clone(),
            loss: self.loss.cohorts.get(filter)?.clone(),
        })
    }

    pub fn aggregate_sources(&self, filter: &Filter) -> Option<UnrealizedAggregateSources> {
        Some(UnrealizedAggregateSources {
            gross_pnl: self.gross_pnl.series.get(filter)?.clone(),
            invested_capital_in_profit: self.invested_capital_in_profit.series.get(filter)?.clone(),
            invested_capital_in_loss: self.invested_capital_in_loss.series.get(filter)?.clone(),
        })
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        rows: &UTXORows<UnrealizedState>,
        spot: Cents,
        aggregate: &UTXOAggregate<UnrealizedState>,
    ) {
        self.profit
            .matrices
            .push(rows.map(|state| state.unrealized_profit));
        self.loss
            .matrices
            .push(rows.map(|state| state.unrealized_loss));
        self.net_pnl.matrices.push(rows.map(|state| {
            CentsSigned::new(
                state.unrealized_profit.inner() as i64 - state.unrealized_loss.inner() as i64,
            )
        }));
        let rows = aggregate.map(|state| UnrealizedAggregateBlockData::new(spot, state));
        self.gross_pnl.push(rows.map(|row| row.gross_pnl));
        self.invested_capital_in_profit
            .push(rows.map(|row| row.invested_capital_in_profit));
        self.invested_capital_in_loss
            .push(rows.map(|row| row.invested_capital_in_loss));
        self.pain_index.push(rows.map(|row| row.pain_index));
        self.greed_index.push(rows.map(|row| row.greed_index));
        self.net_sentiment.push(rows.map(|row| row.net_sentiment));
        self.capitalized_cap_in_profit_raw
            .push(&rows.map(|row| row.capitalized_cap_in_profit_raw));
        self.capitalized_cap_in_loss_raw
            .push(&rows.map(|row| row.capitalized_cap_in_loss_raw));
    }

    pub fn min_resume_len(&self) -> usize {
        self.profit
            .matrices
            .min_len()
            .min(self.loss.matrices.min_len())
            .min(self.net_pnl.matrices.min_len())
            .min(self.gross_pnl.len())
            .min(self.invested_capital_in_profit.len())
            .min(self.invested_capital_in_loss.len())
            .min(self.pain_index.len())
            .min(self.greed_index.len())
            .min(self.net_sentiment.len())
            .min(self.capitalized_cap_in_profit_raw.len())
            .min(self.capitalized_cap_in_loss_raw.len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.profit.matrices.collect_vecs_mut();
        vecs.extend(self.loss.matrices.collect_vecs_mut());
        vecs.extend(self.net_pnl.matrices.collect_vecs_mut());
        vecs.extend([
            self.gross_pnl.stored_mut(),
            self.invested_capital_in_profit.stored_mut(),
            self.invested_capital_in_loss.stored_mut(),
            self.pain_index.stored_mut(),
            self.greed_index.stored_mut(),
            self.net_sentiment.stored_mut(),
            self.capitalized_cap_in_profit_raw.stored_mut(),
            self.capitalized_cap_in_loss_raw.stored_mut(),
        ]);
        vecs
    }
}

struct UnrealizedAggregateBlockData {
    gross_pnl: Cents,
    invested_capital_in_profit: Cents,
    invested_capital_in_loss: Cents,
    capitalized_cap_in_profit_raw: CentsSquaredSats,
    capitalized_cap_in_loss_raw: CentsSquaredSats,
    pain_index: Cents,
    greed_index: Cents,
    net_sentiment: CentsSigned,
}

impl UnrealizedAggregateBlockData {
    #[inline(always)]
    fn new(spot: Cents, state: &UnrealizedState) -> Self {
        let market_value_in_profit =
            state.supply_in_profit.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let market_value_in_loss =
            state.supply_in_loss.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let invested_capital_in_profit = Cents::new(
            market_value_in_profit.saturating_sub(state.unrealized_profit.as_u128()) as u64,
        );
        let invested_capital_in_loss =
            Cents::new((market_value_in_loss + state.unrealized_loss.as_u128()) as u64);
        let greed_index = Cents::new(spot.as_u128().saturating_sub(Self::capitalized_price(
            state.capitalized_cap_in_profit_raw,
            invested_capital_in_profit,
        )) as u64);
        let pain_index = Cents::new(
            Self::capitalized_price(state.capitalized_cap_in_loss_raw, invested_capital_in_loss)
                .saturating_sub(spot.as_u128()) as u64,
        );

        Self {
            gross_pnl: state.unrealized_profit + state.unrealized_loss,
            invested_capital_in_profit,
            invested_capital_in_loss,
            capitalized_cap_in_profit_raw: CentsSquaredSats::new(
                state.capitalized_cap_in_profit_raw,
            ),
            capitalized_cap_in_loss_raw: CentsSquaredSats::new(state.capitalized_cap_in_loss_raw),
            pain_index,
            greed_index,
            net_sentiment: CentsSigned::new(
                i64::try_from(i128::from(greed_index.inner()) - i128::from(pain_index.inner()))
                    .expect("net sentiment overflowed CentsSigned"),
            ),
        }
    }

    #[inline(always)]
    fn capitalized_price(raw: u128, invested_capital: Cents) -> u128 {
        let invested_raw = invested_capital.as_u128() * Sats::ONE_BTC_U128;
        raw.checked_div(invested_raw).unwrap_or_default()
    }
}
