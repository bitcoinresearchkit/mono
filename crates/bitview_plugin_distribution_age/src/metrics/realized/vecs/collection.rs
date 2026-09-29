use bitview_cohort::{
    CohortContext, CohortId, UTXOAggregate, UTXOAggregateId, UTXOAllAndSth, UTXOCoreValues,
    UTXOGroupsWithoutAmountOrType,
};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{
    NegCentsUnsignedToDollars, RatioCents, RatioCentsF32, RatioCentsSignedCents, SoprRatio,
};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlock, LazyPercentPerBlock, LazyWindowStartVec, PercentRollingWindows,
    RollingWindows, RollingWindowsFrom1w, UTXOAgeSources,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{
    Cents, CentsSats, CentsSigned, CentsSquaredSats, Height, PartsPerMillion32,
    PartsPerMillionSigned64, PriceRatio, StoredF32, Version,
};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, Ident, LazyVec, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

use super::{
    super::{
        AdjustedSoprVecs, NegRealizedLoss, RealizedAggregateSources, RealizedAggregateState,
        Sopr24hInput, Sopr24hVecs,
    },
    CumulativeNetRealizedByCohort, CumulativeRealizedByCohort, CumulativeValueDestroyedByCohort,
    RealizedCapByCohort, RealizedPriceByCohort, RealizedSources,
};
use crate::metrics::{
    AdditiveAggregateFiatPerBlockCumulativeWithSums, AggregatePercentPerBlock,
    AggregatePriceWithRatioPerBlock, RealizedBlockData,
};

#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    /// Creation-date value of a UTXO or address-balance cohort's unspent outputs: the sum of each
    /// output's BTC value multiplied by Bitcoin's spot price when that output
    /// was created.
    pub cap: RealizedCapByCohort<M>,
    /// Realized price of a UTXO cohort: the satoshi-weighted mean Bitcoin spot
    /// price at which its currently unspent outputs were created, calculated as
    /// Σ(creation price × unspent sats) / Σ(unspent sats). It is the cohort's
    /// aggregate on-chain cost basis. Returns zero when the cohort has no
    /// unspent supply.
    pub price: RealizedPriceByCohort<M>,
    /// Profit realized by outputs from a UTXO or pre-spend address-balance cohort: spending
    /// value minus creation-date value, counted only for profitable spends.
    pub profit: CumulativeRealizedByCohort<M>,
    /// Loss realized by outputs from a UTXO or pre-spend address-balance cohort:
    /// creation-date value minus spending value, counted only for losing spends.
    pub loss: CumulativeRealizedByCohort<M>,
    /// Net realized profit and loss of outputs from a UTXO cohort when
    /// spent: realized profit minus realized loss.
    pub net_pnl: CumulativeNetRealizedByCohort<M>,
    #[traversable(wrap = "sopr")]
    /// Creation-date value destroyed by spent outputs from a UTXO cohort:
    /// the sum of each spent output's creation price multiplied by its BTC
    /// value.
    pub value_destroyed: CumulativeValueDestroyedByCohort<M>,
    /// 24-hour spent output profit ratio for a UTXO cohort: spending
    /// value divided by creation-date value for outputs spent over the trailing
    /// 24 hours. Values above one mean aggregate profit and values below one
    /// mean aggregate loss. Returns one when creation-date value is zero.
    pub sopr: Sopr24hVecs<M>,
    /// Adjusted spent output profit ratio (SOPR) inputs and ratios for the
    /// all-chain and short-term-holder cohorts after excluding outputs younger
    /// than one hour.
    pub adjusted_sopr: AdjustedSoprVecs<M>,
    /// Gross realized profit and loss of an aggregate UTXO cohort:
    /// realized profit plus realized loss.
    pub gross_pnl: AdditiveAggregateFiatPerBlockCumulativeWithSums<Cents, M>,
    /// Capitalized price of an aggregate UTXO cohort: the mean Bitcoin spot
    /// price at which its currently unspent outputs were created, weighted by
    /// each output's creation-date USD value. It is calculated as Σ(creation
    /// price² × unspent sats) / Σ(creation price × unspent sats), so expensive
    /// acquisitions receive more weight than in realized price. Returns zero
    /// when the cohort has no invested value.
    pub capitalized_price: AggregatePriceWithRatioPerBlock<M>,
    /// Raw sum of creation price in cents per BTC multiplied by unspent
    /// satoshis for a disjoint age band or holder aggregate. Dividing by 100,000,000 converts
    /// it to realized capitalization in cents; dividing by unspent satoshis
    /// gives realized price in cents per BTC. It is an intermediate product,
    /// not itself a capitalization or price.
    pub cap_raw: UTXOAgeSources<CentsSats, M>,
    /// Raw sum of squared creation price in cents per BTC multiplied by unspent
    /// satoshis for a disjoint age band or holder aggregate. Dividing it by the cohort's raw
    /// creation-price-times-satoshis sum gives capitalized price in cents per
    /// BTC. It is an intermediate product, not itself a capitalization or
    /// price.
    pub capitalized_cap_raw: UTXOAgeSources<CentsSquaredSats, M>,
    /// Value forgone relative to each spent output's highest Bitcoin spot price
    /// from its creation block through its spending block, inclusive: that peak
    /// minus the spending price, multiplied by the output's BTC value.
    pub peak_regret: AdditiveAggregateFiatPerBlockCumulativeWithSums<Cents, M>,
    /// Change over the trailing 30-day monotonic-time window in an aggregate
    /// UTXO cohort's cumulative net realized profit and loss, divided by that
    /// cohort's realized cap at the represented block. Positive values mean
    /// cumulative realized profit increased relative to realized loss; negative
    /// values mean the reverse. Returns zero when realized cap is zero.
    pub net_pnl_change_1m_to_rcap: AggregatePercentPerBlock<PartsPerMillionSigned64, M>,
    /// For each supported trailing window, gross realized profit and loss
    /// divided by an aggregate UTXO cohort's realized cap at the represented
    /// block. Larger values mean more capital changed hands far from its
    /// creation price relative to the cohort's invested capital base.
    pub sell_side_risk_ratio: UTXOAggregate<PercentRollingWindows<PartsPerMillion32, M>>,
    /// For each supported trailing window, spent output profit ratio: spending
    /// value divided by creation-date value for outputs spent from an aggregate
    /// UTXO cohort. Values above one mean the outputs were spent in aggregate
    /// profit; values below one mean aggregate loss. Returns one when
    /// creation-date value is zero.
    pub sopr_ratio_extended: UTXOAggregate<RollingWindowsFrom1w<StoredF32, M>>,
    /// For each supported trailing window, realized profit divided by realized
    /// loss for an aggregate UTXO cohort. Values above one mean realized profit
    /// exceeded realized loss in the window; values below one mean the reverse.
    /// Returns one when realized loss is zero.
    pub profit_to_loss_ratio: UTXOAggregate<RollingWindows<StoredF32, M>>,
    /// Market-value-to-realized-value (MVRV) ratio for a UTXO cohort: spot
    /// price divided by its realized price. Values above one place spot above
    /// the cohort's aggregate on-chain cost basis; values below one place it
    /// below that basis.
    pub mvrv: UTXOGroupsWithoutAmountOrType<LazyPerBlock<StoredF32>>,
    #[traversable(wrap = "loss", rename = "negative")]
    /// Loss realized by outputs from a UTXO cohort when spent, expressed as a
    /// negative value.
    pub negative_loss: UTXOGroupsWithoutAmountOrType<NegRealizedLoss>,
    #[traversable(wrap = "cap", rename = "to_own_mcap")]
    /// Realized cap divided by an aggregate UTXO cohort's own market cap,
    /// equivalently realized price divided by spot price and the reciprocal of
    /// MVRV. Values above one place spot below the cohort's aggregate on-chain
    /// cost basis; values below one place it above that basis.
    pub cap_to_own_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "net_pnl/change_1m", rename = "to_mcap")]
    /// Change over the trailing 30-day monotonic-time window in an aggregate
    /// UTXO cohort's cumulative net realized profit and loss, divided by total
    /// Bitcoin market cap at the represented block. Positive values mean
    /// cumulative realized profit increased relative to realized loss; negative
    /// values mean the reverse. Returns zero when market cap is zero.
    pub net_pnl_change_1m_to_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}

impl RealizedVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_market_cap: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Box<Self>> {
        let aggregate_version = version + Version::ONE;
        let gross_pnl = AdditiveAggregateFiatPerBlockCumulativeWithSums::forced_import(
            db,
            "realized_gross_pnl",
            aggregate_version,
            mappings,
            window_starts,
        )?;
        let capitalized_price = AggregatePriceWithRatioPerBlock::forced_import(
            db,
            "capitalized_price",
            aggregate_version,
            mappings,
            spot_price,
        )?;
        let cap_raw = UTXOAgeSources::forced_import(db, "cap_raw", version)?;
        let capitalized_cap_raw =
            UTXOAgeSources::forced_import(db, "capitalized_cap_raw", version)?;
        let peak_regret = AdditiveAggregateFiatPerBlockCumulativeWithSums::forced_import(
            db,
            "realized_peak_regret",
            aggregate_version,
            mappings,
            window_starts,
        )?;
        let net_pnl_change_1m_to_rcap = AggregatePercentPerBlock::forced_import(
            db,
            "net_pnl_change_1m_to_rcap",
            aggregate_version,
            mappings,
        )?;
        let sell_side_risk_ratio = UTXOAggregate::try_from_fn(|id| {
            PercentRollingWindows::forced_import(
                db,
                &id.metric_name("sell_side_risk_ratio"),
                Self::aggregate_metric_version(version, id, Version::TWO),
                mappings,
            )
        })?;
        let sopr_ratio_extended = UTXOAggregate::try_from_fn(|id| {
            RollingWindowsFrom1w::forced_import(
                db,
                &id.metric_name("sopr"),
                Self::aggregate_metric_version(version, id, Version::TWO),
                mappings,
            )
        })?;
        let profit_to_loss_ratio = UTXOAggregate::try_from_fn(|id| {
            RollingWindows::forced_import(
                db,
                &id.metric_name("realized_profit_to_loss_ratio"),
                Self::aggregate_metric_version(version, id, Version::TWO),
                mappings,
            )
        })?;
        let cap = RealizedCapByCohort::forced_import(db, version, mappings, window_starts)?;
        let price = RealizedPriceByCohort::forced_import(db, version, mappings, spot_price)?;
        let profit = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_profit",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let loss = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_loss",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let net_pnl =
            CumulativeNetRealizedByCohort::forced_import(db, version, mappings, window_starts)?;
        let value_destroyed = CumulativeValueDestroyedByCohort::forced_import(
            db,
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let sopr = Sopr24hVecs::forced_import(db, version, mappings)?;
        let adjusted_sopr = AdjustedSoprVecs::forced_import(db, version, mappings, window_starts)?;
        let mvrv = price.cohorts.map_with_id(|cohort_id, price| {
            LazyPerBlock::from_lazy::<Ident, PriceRatio>(
                &CohortContext::Utxo.metric_name(cohort_id, "mvrv"),
                Self::cohort_version(version, cohort_id),
                &price.relative.ratio,
            )
        });
        let negative_loss = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let loss = loss.cohorts.get(cohort_id).expect("realized-loss cohort");
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_loss_neg");
            let version = Self::cohort_version(version, cohort_id) + Version::ONE;
            let base = LazyVec::transformed::<NegCentsUnsignedToDollars>(
                &name,
                version,
                loss.block.cents.read_only_boxed_clone(),
            );
            let sum = loss.sum.0.map_with_suffix(|suffix, slot| {
                let source = slot.cents.height.clone();
                LazyPerBlock::from_height_source::<NegCentsUnsignedToDollars>(
                    &format!("{name}_sum_{suffix}"),
                    version,
                    &source,
                    mappings,
                )
            });
            NegRealizedLoss { base, sum }
        });
        let cap_to_own_mcap = UTXOAggregate::from_fn(|id| {
            let cohort_id = id.cohort();
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_cap_to_own_mcap");
            let source = LazyVec::init(
                &format!("{name}_ppm_source"),
                Self::cohort_version(version, cohort_id) + Version::TWO,
                price
                    .cohorts
                    .get(cohort_id)
                    .expect("realized-price cohort")
                    .relative
                    .ppm
                    .height
                    .read_only_boxed_clone(),
                Self::mvrv_to_realized_cap_ratio,
            );
            LazyPercentPerBlock::from_height_source(
                &name,
                Self::cohort_version(version, cohort_id) + Version::TWO,
                &source,
                mappings,
            )
        });
        let net_pnl_change_1m_to_mcap = UTXOAggregate::from_fn(|id| {
            let cohort_id = id.cohort();
            let name = CohortContext::Utxo.metric_name(cohort_id, "net_pnl_change_1m_to_mcap");
            let source = LazyIndexedVec::new(
                &format!("{name}_ppm_source"),
                Version::new(5),
                &net_pnl
                    .cohorts
                    .get(cohort_id)
                    .expect("aggregate net-realized-PnL cohort")
                    .delta
                    .absolute
                    ._1m
                    .cents
                    .height,
                all_market_cap,
                |_, net_pnl, market_cap| Self::net_pnl_to_market_cap(net_pnl, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(5), &source, mappings)
        });

        Ok(Box::new(Self {
            cap,
            price,
            profit,
            loss,
            net_pnl,
            value_destroyed,
            sopr,
            adjusted_sopr,
            gross_pnl,
            capitalized_price,
            cap_raw,
            capitalized_cap_raw,
            peak_regret,
            net_pnl_change_1m_to_rcap,
            sell_side_risk_ratio,
            sopr_ratio_extended,
            profit_to_loss_ratio,
            mvrv,
            negative_loss,
            cap_to_own_mcap,
            net_pnl_change_1m_to_mcap,
        }))
    }

    fn cohort_version(version: Version, cohort_id: CohortId) -> Version {
        version
            + if matches!(cohort_id, CohortId::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    fn aggregate_metric_version(version: Version, id: UTXOAggregateId, offset: Version) -> Version {
        version
            + offset
            + if matches!(id, UTXOAggregateId::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    fn net_pnl_to_market_cap(net_pnl: CentsSigned, market_cap: Cents) -> PartsPerMillionSigned64 {
        let market_cap = f64::from(market_cap);
        if market_cap > 0.0 {
            PartsPerMillionSigned64::from(net_pnl.inner() as f64 / market_cap)
        } else {
            PartsPerMillionSigned64::default()
        }
    }

    #[inline(always)]
    fn mvrv_to_realized_cap_ratio(_: Height, mvrv: PriceRatio) -> PartsPerMillion32 {
        PartsPerMillion32::from(1.0 / f64::from(mvrv))
    }

    pub fn sources(&self, cohort_id: CohortId) -> Option<RealizedSources> {
        Some(RealizedSources {
            cap: self.cap.cohorts.get(cohort_id)?.clone(),
            profit: self.profit.cohorts.get(cohort_id)?.clone(),
            loss: self.loss.cohorts.get(cohort_id)?.clone(),
            net_pnl: self.net_pnl.cohorts.get(cohort_id)?.clone(),
            value_destroyed: self.value_destroyed.cohorts.get(cohort_id)?.clone(),
        })
    }

    #[inline(always)]
    pub fn push_aggregate(
        &mut self,
        cohort_values: &UTXOAggregate<RealizedAggregateState>,
    ) -> Cents {
        let prices = cohort_values.map(RealizedAggregateState::capitalized_price);
        self.gross_pnl
            .push_block(cohort_values.map(RealizedAggregateState::gross_pnl));
        self.capitalized_price.push(prices.clone());
        self.peak_regret
            .push_block(cohort_values.map(RealizedAggregateState::peak_regret));
        self.cap_raw
            .push(&cohort_values.map(|values| values.cap_raw));
        self.capitalized_cap_raw
            .push(&cohort_values.map(|values| values.capitalized_cap_raw));
        prices.all
    }

    pub fn compute_sopr(
        &mut self,
        max_from: Height,
        inputs: &UTXOGroupsWithoutAmountOrType<Sopr24hInput>,
        exit: &Exit,
    ) -> Result<()> {
        self.sopr.compute(max_from, inputs, exit)
    }

    pub fn compute_adjusted_sopr<V1, V2>(
        &mut self,
        max_from: Height,
        sources: &UTXOAllAndSth<&RealizedAggregateSources>,
        under_1h_transfer_volume_cumulative: &V1,
        under_1h_value_destroyed_cumulative: &V2,
        exit: &Exit,
    ) -> Result<()>
    where
        V1: ReadableVec<Height, Cents>,
        V2: ReadableVec<Height, Cents>,
    {
        self.adjusted_sopr.compute(
            max_from,
            sources,
            under_1h_transfer_volume_cumulative,
            under_1h_value_destroyed_cumulative,
            exit,
        )
    }

    pub fn compute_aggregate_metrics(
        &mut self,
        max_from: Height,
        sources: &UTXOAggregate<RealizedAggregateSources>,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            gross_pnl,
            net_pnl_change_1m_to_rcap,
            sell_side_risk_ratio,
            sopr_ratio_extended,
            profit_to_loss_ratio,
            ..
        } = self;

        for id in UTXOAggregateId::ALL {
            let source = id.select(sources);
            let realized = &source.realized;

            id.select_mut(&mut net_pnl_change_1m_to_rcap.stored)
                .compute_transform2(
                    max_from,
                    &realized.net_pnl.delta.absolute._1m.cents.height,
                    &realized.cap.cents.height,
                    |(height, change, cap, _)| {
                        (
                            height,
                            RatioCentsSignedCents::<PartsPerMillionSigned64>::apply(change, cap),
                        )
                    },
                    exit,
                )?;

            for ((target, created), destroyed) in id
                .select_mut(sopr_ratio_extended)
                .as_mut_array()
                .into_iter()
                .zip(
                    source
                        .activity
                        .transfer_volume
                        .sum
                        .0
                        .as_array()
                        .into_iter()
                        .skip(1),
                )
                .zip(realized.value_destroyed.sum.as_array().into_iter().skip(1))
            {
                target.compute_binary::<_, _, SoprRatio>(
                    max_from,
                    &created.cents.height,
                    &destroyed.cents.height,
                    exit,
                )?;
            }
            for (target, pnl) in id
                .select_mut(sell_side_risk_ratio)
                .as_mut_array()
                .into_iter()
                .zip(id.select(&gross_pnl.series).sum.as_array())
            {
                target.compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                    max_from,
                    &pnl.cents.height,
                    &realized.cap.cents.height,
                    exit,
                )?;
            }
            for ((target, profit), loss) in id
                .select_mut(profit_to_loss_ratio)
                .as_mut_array()
                .into_iter()
                .zip(realized.profit.sum.as_array())
                .zip(realized.loss.sum.as_array())
            {
                target.compute_binary::<_, _, RatioCentsF32>(
                    max_from,
                    &profit.cents.height,
                    &loss.cents.height,
                    exit,
                )?;
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: &UTXOCoreValues<RealizedBlockData>) {
        let totals = cohort_values.map(RealizedBlockData::totals);
        let aggregate_price = UTXOAggregate::from_fn(|id| {
            totals.value(id.cohort()).expect("aggregate cohort").price()
        });

        self.cap.stored.push(cohort_values.map(|values| values.cap));
        self.price
            .stored
            .push_exact(cohort_values.map(|values| values.price), aggregate_price);
        self.profit
            .stored
            .push_block(cohort_values.map(|values| values.profit));
        self.loss
            .stored
            .push_block(cohort_values.map(|values| values.loss));
        self.net_pnl
            .stored
            .push_block(cohort_values.map(|values| values.net_pnl));
        self.value_destroyed
            .stored
            .push_block(cohort_values.map(|values| values.value_destroyed));
    }

    /// Only values pushed during block processing belong here. SOPR and the
    /// other ratios are rebuilt afterward from these stored sources.
    pub fn min_resume_len(&self) -> usize {
        self.cap
            .stored
            .min_len()
            .min(self.price.stored.min_len())
            .min(self.profit.stored.min_len())
            .min(self.loss.stored.min_len())
            .min(self.net_pnl.stored.min_len())
            .min(self.value_destroyed.stored.min_len())
            .min(self.gross_pnl.len())
            .min(self.capitalized_price.len())
            .min(self.peak_regret.len())
            .min(self.cap_raw.len())
            .min(self.capitalized_cap_raw.len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.cap.stored.collect_vecs_mut();
        vecs.extend(self.price.stored.collect_vecs_mut());
        vecs.extend(self.profit.stored.collect_vecs_mut());
        vecs.extend(self.loss.stored.collect_vecs_mut());
        vecs.extend(self.net_pnl.stored.collect_vecs_mut());
        vecs.extend(self.value_destroyed.stored.collect_vecs_mut());
        vecs.extend(self.sopr.collect_vecs_mut());
        vecs.extend(self.adjusted_sopr.collect_vecs_mut());
        vecs.extend(self.gross_pnl.collect_vecs_mut());
        vecs.extend(self.capitalized_price.collect_vecs_mut());
        vecs.extend(self.peak_regret.collect_vecs_mut());
        vecs.extend(self.net_pnl_change_1m_to_rcap.collect_vecs_mut());
        vecs.extend(self.cap_raw.collect_vecs_mut());
        vecs.extend(self.capitalized_cap_raw.collect_vecs_mut());
        vecs.extend(
            self.sell_side_risk_ratio
                .iter_mut()
                .flat_map(|value| value.as_mut_array())
                .map(|value| &mut value.ppm.height as &mut dyn AnyStoredVec),
        );
        vecs.extend(
            self.sopr_ratio_extended
                .iter_mut()
                .flat_map(|value| value.as_mut_array())
                .map(|value| &mut value.height as &mut dyn AnyStoredVec),
        );
        vecs.extend(
            self.profit_to_loss_ratio
                .iter_mut()
                .flat_map(|value| value.as_mut_array())
                .map(|value| &mut value.height as &mut dyn AnyStoredVec),
        );
        vecs
    }
}

#[cfg(test)]
mod price_ratio_tests {
    use super::*;

    #[test]
    fn inverse_mvrv_keeps_saturated_values_finite() {
        let inverse = |ratio| RealizedVecs::mvrv_to_realized_cap_ratio(Height::from(0usize), ratio);
        assert_eq!(inverse(PriceRatio::from(2.0)), PartsPerMillion32::from(0.5));
        assert_eq!(
            inverse(PriceRatio::from(10_000.0)),
            PartsPerMillion32::new(233)
        );
        assert!(inverse(PriceRatio::NAN).is_nan());
        assert!(inverse(PriceRatio::ZERO).is_nan());
    }
}
