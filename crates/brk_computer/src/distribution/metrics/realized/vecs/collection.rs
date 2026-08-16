use brk_cohort::{
    AmountRange, CohortContext, Filter, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES,
    UTXOAggregate, UTXOAggregateId, UTXOAllAndSth, UTXOGroups, UTXOGroupsWithoutAmountOrType,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    Cents, CentsSats, CentsSigned, CentsSquaredSats, Height, PartsPerMillion32, PartsPerMillion64,
    PartsPerMillionSigned64, StoredF32, StoredF64, Version,
};
use vecdb::{
    AnyStoredVec, BinaryTransform, CachedBoxedVec, ColumnId, Database, Exit, LazyVec,
    ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

use crate::{
    distribution::{
        AllChainSources,
        metrics::{
            AdditiveAggregateFiatPerBlockCumulativeWithSums, AdditiveUTXORawVec,
            AggregatePercentPerBlock, AggregatePriceWithRatioPerBlock, ColumnarAmount,
            RealizedBlockData, RealizedTotals, UTXORows,
        },
    },
    indexes,
    internal::{
        CACHE_BUDGET, CachedWindowStartVec, ColumnarPercentRollingWindows, ColumnarRollingWindows,
        ColumnarRollingWindowsFrom1w, Identity, LazyFiatPerBlockCumulativeWithSums,
        LazyFiatPerBlockWithDeltas, LazyPerBlock, LazyPercentPerBlock, NegCentsUnsignedToDollars,
        RatioCents, RatioCents64, RatioCentsSignedCents, SoprRatio, Windows,
    },
};

use super::{
    super::{
        AdjustedSoprComputeSource, AdjustedSoprVecs, NegRealizedLoss, RealizedAggregateSources,
        RealizedAggregateState, Sopr24hInput, Sopr24hVecs,
    },
    CumulativeNetRealizedByCohort, CumulativeRealizedByCohort, CumulativeValueDestroyedByCohort,
    RealizedCapByCohort, RealizedPriceByCohort, RealizedSources,
};

#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    pub cap: RealizedCapByCohort<M>,
    /// The sats-weighted average USD creation price of the unspent outputs in
    /// the selected cohort: Σ(creation price × unspent sats) / Σ(unspent sats).
    /// Returns zero when the cohort has no unspent supply.
    pub price: RealizedPriceByCohort<M>,
    pub profit: CumulativeRealizedByCohort<M>,
    #[traversable(wrap = "profit", rename = "addr_balance")]
    pub addr_balance_profit: ColumnarAmount<Cents, LazyFiatPerBlockCumulativeWithSums<Cents>, M>,
    pub loss: CumulativeRealizedByCohort<M>,
    #[traversable(wrap = "loss", rename = "addr_balance")]
    pub addr_balance_loss: ColumnarAmount<Cents, LazyFiatPerBlockCumulativeWithSums<Cents>, M>,
    pub net_pnl: CumulativeNetRealizedByCohort<M>,
    #[traversable(wrap = "sopr")]
    pub value_destroyed: CumulativeValueDestroyedByCohort<M>,
    pub sopr: Sopr24hVecs<M>,
    pub adjusted_sopr: AdjustedSoprVecs<M>,
    #[traversable(wrap = "cap", rename = "addr_balance")]
    pub addr_balance_cap: ColumnarAmount<
        Cents,
        LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
        M,
    >,
    pub gross_pnl: AdditiveAggregateFiatPerBlockCumulativeWithSums<Cents, M>,
    pub capitalized_price: AggregatePriceWithRatioPerBlock<M>,
    pub cap_raw: AdditiveUTXORawVec<CentsSats, M>,
    pub capitalized_cap_raw: AdditiveUTXORawVec<CentsSquaredSats, M>,
    pub peak_regret: AdditiveAggregateFiatPerBlockCumulativeWithSums<Cents, M>,
    pub net_pnl_change_1m_to_rcap: AggregatePercentPerBlock<PartsPerMillionSigned64, M>,
    pub sell_side_risk_ratio: UTXOAggregate<ColumnarPercentRollingWindows<PartsPerMillion32, M>>,
    pub sopr_ratio_extended: UTXOAggregate<ColumnarRollingWindowsFrom1w<StoredF32, M>>,
    pub profit_to_loss_ratio: UTXOAggregate<ColumnarRollingWindows<StoredF64, M>>,
    pub mvrv: UTXOGroups<LazyPerBlock<StoredF32>>,
    #[traversable(wrap = "loss", rename = "negative")]
    pub negative_loss: UTXOGroupsWithoutAmountOrType<NegRealizedLoss>,
    #[traversable(wrap = "cap", rename = "to_own_mcap")]
    pub cap_to_own_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "net_pnl/change_1m", rename = "to_mcap")]
    pub net_pnl_change_1m_to_mcap: UTXOAggregate<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}

impl RealizedVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        let aggregate_version = version + Version::ONE;
        let gross_pnl = AdditiveAggregateFiatPerBlockCumulativeWithSums::forced_import(
            db,
            "realized_gross_pnl",
            aggregate_version,
            indexes,
            cached_starts,
        )?;
        let capitalized_price = AggregatePriceWithRatioPerBlock::forced_import(
            db,
            "capitalized_price",
            aggregate_version,
            indexes,
            spot_price,
        )?;
        let cap_raw = AdditiveUTXORawVec::forced_import(db, "cap_raw", version)?;
        let capitalized_cap_raw =
            AdditiveUTXORawVec::forced_import(db, "capitalized_cap_raw", version)?;
        let peak_regret = AdditiveAggregateFiatPerBlockCumulativeWithSums::forced_import(
            db,
            "realized_peak_regret",
            aggregate_version,
            indexes,
            cached_starts,
        )?;
        let net_pnl_change_1m_to_rcap = AggregatePercentPerBlock::forced_import(
            db,
            "net_pnl_change_1m_to_rcap",
            aggregate_version,
            indexes,
        )?;
        let sell_side_risk_ratio = UTXOAggregate::try_from_fn(|id| {
            ColumnarPercentRollingWindows::forced_import(
                db,
                &Self::aggregate_metric_name(id, "sell_side_risk_ratio"),
                Self::aggregate_metric_version(version, id, Version::TWO),
                indexes,
            )
        })?;
        let sopr_ratio_extended = UTXOAggregate::try_from_fn(|id| {
            ColumnarRollingWindowsFrom1w::forced_import(
                db,
                &Self::aggregate_metric_name(id, "sopr"),
                Self::aggregate_metric_version(version, id, Version::TWO),
                indexes,
            )
        })?;
        let profit_to_loss_ratio = UTXOAggregate::try_from_fn(|id| {
            ColumnarRollingWindows::forced_import(
                db,
                &Self::aggregate_metric_name(id, "realized_profit_to_loss_ratio"),
                Self::aggregate_metric_version(version, id, Version::ONE),
                indexes,
            )
        })?;
        let cap = RealizedCapByCohort::forced_import(db, version, indexes, cached_starts)?;
        let price = RealizedPriceByCohort::forced_import(db, version, indexes, spot_price)?;
        let profit = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_profit",
            version + Version::ONE,
            indexes,
            cached_starts,
        )?;
        let loss = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_loss",
            version + Version::ONE,
            indexes,
            cached_starts,
        )?;
        let net_pnl =
            CumulativeNetRealizedByCohort::forced_import(db, version, indexes, cached_starts)?;
        let value_destroyed = CumulativeValueDestroyedByCohort::forced_import(
            db,
            version + Version::ONE,
            indexes,
            cached_starts,
        )?;
        let sopr = Sopr24hVecs::forced_import(db, version, indexes)?;
        let adjusted_sopr = AdjustedSoprVecs::forced_import(db, version, indexes, cached_starts)?;
        let addr_version = version + Version::ONE;
        let addr_balance_cap = ColumnarAmount::forced_import(
            db,
            "addrs_realized_cap_cents_by_balance_range",
            CohortContext::Addr,
            "realized_cap",
            addr_version,
            |name, source| {
                LazyFiatPerBlockWithDeltas::from_boxed_cents_source(
                    name,
                    addr_version,
                    source,
                    Version::TWO,
                    indexes,
                    cached_starts,
                )
            },
        )?;
        let addr_balance_profit = ColumnarAmount::forced_import(
            db,
            "addrs_realized_profit_cumulative_cents_by_balance_range",
            CohortContext::Addr,
            "realized_profit",
            addr_version + Version::ONE,
            |name, source| {
                LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
                    name,
                    addr_version + Version::ONE,
                    source,
                    indexes,
                    cached_starts,
                )
            },
        )?;
        let addr_balance_loss = ColumnarAmount::forced_import(
            db,
            "addrs_realized_loss_cumulative_cents_by_balance_range",
            CohortContext::Addr,
            "realized_loss",
            addr_version + Version::ONE,
            |name, source| {
                LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
                    name,
                    addr_version + Version::ONE,
                    source,
                    indexes,
                    cached_starts,
                )
            },
        )?;

        let mvrv = price.cohorts.map_named(|filter, cohort_name, price| {
            LazyPerBlock::from_lazy::<Identity<StoredF32>, PartsPerMillion64>(
                &CohortContext::Utxo.metric_name(filter, cohort_name, "mvrv"),
                Self::cohort_version(version, filter),
                &price.ratio,
            )
        });
        let negative_loss = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let loss = loss.cohorts.get(&filter).expect("realized-loss cohort");
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "realized_loss_neg");
            let version = Self::cohort_version(version, &filter) + Version::ONE;
            let base = LazyVec::transformed::<NegCentsUnsignedToDollars>(
                &name,
                version,
                loss.block.cents.read_only_boxed_clone(),
            );
            let sum = loss.sum.0.map_with_suffix(|suffix, slot| {
                let source = CACHE_BUDGET.wrap(slot.cents.height.clone());
                LazyPerBlock::from_height_source::<NegCentsUnsignedToDollars>(
                    &format!("{name}_sum_{suffix}"),
                    version,
                    source,
                    indexes,
                )
            });
            NegRealizedLoss { base, sum }
        });
        let cap_to_own_mcap = UTXOAggregate::from_fn(|id| {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            let name = CohortContext::Utxo.metric_name(
                filter,
                id.select(&UTXO_AGGREGATE_NAMES).id,
                "realized_cap_to_own_mcap",
            );
            let source = LazyVec::init(
                &format!("{name}_ppm_source"),
                Self::cohort_version(version, filter) + Version::TWO,
                price
                    .cohorts
                    .get(filter)
                    .expect("realized-price cohort")
                    .ppm
                    .height
                    .read_only_boxed_clone(),
                Self::mvrv_to_realized_cap_ratio,
            );
            let source = CACHE_BUDGET.wrap(source);
            LazyPercentPerBlock::from_height_source(
                &name,
                Self::cohort_version(version, filter) + Version::TWO,
                source,
                indexes,
            )
        });
        let net_pnl_change_1m_to_mcap = UTXOAggregate::from_fn(|id| {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            let name = CohortContext::Utxo.metric_name(
                filter,
                id.select(&UTXO_AGGREGATE_NAMES).id,
                "net_pnl_change_1m_to_mcap",
            );
            let source = all_chain.with_market_cap(
                &format!("{name}_ppm_source"),
                Version::new(5),
                &net_pnl
                    .cohorts
                    .get(filter)
                    .expect("aggregate net-realized-PnL cohort")
                    .delta
                    .absolute
                    ._1m
                    .cents
                    .height,
                |_, net_pnl, market_cap| Self::net_pnl_to_market_cap(net_pnl, market_cap),
            );
            LazyPercentPerBlock::from_height_source(&name, Version::new(5), source, indexes)
        });

        Ok(Self {
            cap,
            price,
            profit,
            addr_balance_profit,
            loss,
            addr_balance_loss,
            net_pnl,
            value_destroyed,
            sopr,
            adjusted_sopr,
            addr_balance_cap,
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

    fn aggregate_metric_version(version: Version, id: UTXOAggregateId, offset: Version) -> Version {
        version
            + offset
            + if matches!(id, UTXOAggregateId::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    fn aggregate_metric_name(id: UTXOAggregateId, metric: &str) -> String {
        CohortContext::Utxo.metric_name(
            id.select(&UTXO_AGGREGATE_FILTERS),
            id.select(&UTXO_AGGREGATE_NAMES).id,
            metric,
        )
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
    fn mvrv_to_realized_cap_ratio(_: Height, mvrv: PartsPerMillion64) -> PartsPerMillion32 {
        PartsPerMillion32::from(1.0 / f64::from(mvrv))
    }

    pub fn sources(&self, filter: &Filter) -> Option<RealizedSources> {
        Some(RealizedSources {
            cap: self.cap.cohorts.get(filter)?.clone(),
            profit: self.profit.cohorts.get(filter)?.clone(),
            loss: self.loss.cohorts.get(filter)?.clone(),
            net_pnl: self.net_pnl.cohorts.get(filter)?.clone(),
            value_destroyed: self.value_destroyed.cohorts.get(filter)?.clone(),
        })
    }

    #[inline(always)]
    pub fn push_aggregate(&mut self, rows: &UTXOAggregate<RealizedAggregateState>) -> Cents {
        let prices = rows.map(RealizedAggregateState::capitalized_price);
        self.gross_pnl
            .push_block(rows.map(RealizedAggregateState::gross_pnl));
        self.capitalized_price.push(prices.clone());
        self.peak_regret
            .push_block(rows.map(RealizedAggregateState::peak_regret));
        self.cap_raw.push(&rows.map(|row| row.cap_raw));
        self.capitalized_cap_raw
            .push(&rows.map(|row| row.capitalized_cap_raw));
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
        sources: &UTXOAllAndSth<AdjustedSoprComputeSource>,
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

        net_pnl_change_1m_to_rcap.compute_columns2(
            max_from,
            |id| {
                &id.select(sources)
                    .realized
                    .net_pnl
                    .delta
                    .absolute
                    ._1m
                    .cents
                    .height
            },
            |id| &id.select(sources).realized.cap.cents.height,
            |_, change, cap| RatioCentsSignedCents::<PartsPerMillionSigned64>::apply(change, cap),
            exit,
        )?;

        for id in UTXOAggregateId::ALL {
            let source = id.select(sources);
            let realized = &source.realized;

            id.select_mut(sopr_ratio_extended).compute_columns2(
                max_from,
                |window| {
                    &window
                        .select_full(&source.activity.transfer_volume.sum.0)
                        .cents
                        .height
                },
                |window| {
                    &window
                        .select_full(&realized.value_destroyed.sum)
                        .cents
                        .height
                },
                |_, value_created, value_destroyed| {
                    SoprRatio::apply(value_created, value_destroyed)
                },
                exit,
            )?;

            id.select_mut(sell_side_risk_ratio).compute_columns2(
                max_from,
                |window| {
                    &window
                        .select(&id.select(&gross_pnl.series).sum)
                        .cents
                        .height
                },
                |_| &realized.cap.cents.height,
                |_, realized_value, realized_cap| {
                    RatioCents::<PartsPerMillion32>::apply(realized_value, realized_cap)
                },
                exit,
            )?;

            id.select_mut(profit_to_loss_ratio).compute_columns2(
                max_from,
                |window| &window.select(&realized.profit.sum).cents.height,
                |window| &window.select(&realized.loss.sum).cents.height,
                |_, profit, loss| RatioCents64::apply(profit, loss),
                exit,
            )?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn push(&mut self, rows: &UTXORows<RealizedBlockData>) {
        let aggregate_price = rows
            .map(RealizedBlockData::totals)
            .aggregate()
            .map(RealizedTotals::price);

        self.cap.matrices.push(rows.map(|values| values.cap));
        self.price
            .matrices
            .push(rows.map(|values| values.price), aggregate_price);
        self.profit
            .cumulative
            .push_block(rows.map(|values| values.profit));
        self.loss
            .cumulative
            .push_block(rows.map(|values| values.loss));
        self.net_pnl
            .cumulative
            .push_block(rows.map(|values| values.net_pnl));
        self.value_destroyed
            .cumulative
            .push_block(rows.map(|values| values.value_destroyed));
    }

    #[inline(always)]
    pub fn push_addr_balance(
        &mut self,
        cap: AmountRange<Cents>,
        profit: &AmountRange<Cents>,
        loss: &AmountRange<Cents>,
    ) {
        self.addr_balance_cap.push(cap);
        self.addr_balance_profit.push_cumulative(profit);
        self.addr_balance_loss.push_cumulative(loss);
    }

    /// Only values pushed during block processing belong here. SOPR and the
    /// other ratios are rebuilt afterward from these stored sources.
    pub fn min_resume_len(&self) -> usize {
        self.cap
            .matrices
            .min_len()
            .min(self.price.matrices.min_len())
            .min(self.profit.cumulative.min_len())
            .min(self.loss.cumulative.min_len())
            .min(self.net_pnl.cumulative.min_len())
            .min(self.value_destroyed.cumulative.min_len())
            .min(self.addr_balance_cap.len())
            .min(self.addr_balance_profit.len())
            .min(self.addr_balance_loss.len())
            .min(self.gross_pnl.len())
            .min(self.capitalized_price.len())
            .min(self.peak_regret.len())
            .min(self.cap_raw.len())
            .min(self.capitalized_cap_raw.len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.cap.matrices.collect_vecs_mut();
        vecs.push(self.addr_balance_cap.stored_mut());
        vecs.extend(self.price.matrices.collect_vecs_mut());
        vecs.extend(self.profit.cumulative.collect_vecs_mut());
        vecs.push(self.addr_balance_profit.stored_mut());
        vecs.extend(self.loss.cumulative.collect_vecs_mut());
        vecs.push(self.addr_balance_loss.stored_mut());
        vecs.extend(self.net_pnl.cumulative.collect_vecs_mut());
        vecs.extend(self.value_destroyed.cumulative.collect_vecs_mut());
        vecs.extend(self.sopr.collect_vecs_mut());
        vecs.extend(self.adjusted_sopr.collect_vecs_mut());
        vecs.extend([
            self.gross_pnl.stored_mut(),
            self.capitalized_price.stored_mut(),
            self.peak_regret.stored_mut(),
            self.net_pnl_change_1m_to_rcap.stored_mut(),
            self.cap_raw.stored_mut(),
            self.capitalized_cap_raw.stored_mut(),
        ]);
        vecs.extend(
            self.sell_side_risk_ratio
                .iter_mut()
                .map(|value| value.stored_mut()),
        );
        vecs.extend(
            self.sopr_ratio_extended
                .iter_mut()
                .map(|value| value.stored_mut()),
        );
        vecs.extend(
            self.profit_to_loss_ratio
                .iter_mut()
                .map(|value| value.stored_mut()),
        );
        vecs
    }
}
