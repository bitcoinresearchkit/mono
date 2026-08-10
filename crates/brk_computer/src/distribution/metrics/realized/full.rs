use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{
    Bitcoin, Cents, CentsSats, CentsSigned, CentsSquaredSats, Height, PartsPerMillion32,
    PartsPerMillion64, PartsPerMillionSigned64, StoredF64, Version,
};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, BytesVec, Exit, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use crate::{
    distribution::AllChainCache,
    distribution::state::{CohortState, CostBasisData, RealizedState, WithCapital},
    internal::{
        ColumnarPercentRollingWindows, ColumnarRollingWindows, ColumnarRollingWindowsFrom1w,
        FiatPerBlockCumulativeWithSums, LazyPercentPerBlock, PercentPerBlock,
        PriceWithRatioPerBlock, RatioCents, RatioCents64, RatioCentsSignedCents,
        ValuePerBlockCumulativeRolling,
    },
    price,
};

use crate::distribution::metrics::ImportConfig;

use super::RealizedCore;

#[derive(Traversable)]
pub struct RealizedNetPnl<M: StorageMode = Rw> {
    #[traversable(wrap = "change_1m", rename = "to_rcap")]
    pub change_1m_to_rcap: PercentPerBlock<PartsPerMillionSigned64, M>,
    #[traversable(wrap = "change_1m", rename = "to_mcap")]
    pub change_1m_to_mcap: LazyPercentPerBlock<PartsPerMillionSigned64>,
}

#[derive(Traversable)]
pub struct RealizedSopr<M: StorageMode = Rw> {
    #[traversable(rename = "ratio")]
    pub ratio_extended: ColumnarRollingWindowsFrom1w<StoredF64, M>,
}

#[derive(Traversable)]
pub struct RealizedPeakRegret<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: FiatPerBlockCumulativeWithSums<Cents, M>,
}

#[derive(Traversable)]
pub struct RealizedCapitalized<M: StorageMode = Rw> {
    pub price: PriceWithRatioPerBlock<M>,
    #[traversable(hidden)]
    cap_raw: M::Stored<BytesVec<Height, CentsSquaredSats>>,
}

#[derive(Deref, DerefMut, Traversable)]
pub struct RealizedFull<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub core: RealizedCore<M>,

    #[traversable(wrap = "cap", rename = "to_own_mcap")]
    pub cap_to_own_mcap: LazyPercentPerBlock<PartsPerMillion32>,
    pub gross_pnl: FiatPerBlockCumulativeWithSums<Cents, M>,
    pub sell_side_risk_ratio: ColumnarPercentRollingWindows<PartsPerMillion32, M>,
    pub net_pnl: RealizedNetPnl<M>,
    pub sopr: RealizedSopr<M>,
    pub peak_regret: RealizedPeakRegret<M>,
    pub capitalized: RealizedCapitalized<M>,

    pub profit_to_loss_ratio: ColumnarRollingWindows<StoredF64, M>,

    #[traversable(hidden)]
    cap_raw: M::Stored<BytesVec<Height, CentsSats>>,
}

impl RealizedFull {
    pub(crate) fn forced_import(cfg: &ImportConfig, all_chain: &AllChainCache) -> Result<Self> {
        let v0 = Version::ZERO;
        let v1 = Version::ONE;

        let core = RealizedCore::forced_import(cfg)?;
        let cap_to_own_mcap = LazyPercentPerBlock::from_indexed_source(
            &cfg.name("realized_cap_to_own_mcap"),
            cfg.version + Version::TWO,
            &core.minimal.price.ppm.height,
            mvrv_to_realized_cap_ratio,
            cfg.indexes,
        );

        // Gross PnL
        let gross_pnl: FiatPerBlockCumulativeWithSums<Cents> =
            cfg.import("realized_gross_pnl", v1)?;
        let sell_side_risk_ratio = cfg.import("sell_side_risk_ratio", Version::new(2))?;

        // Net PnL
        let mcap_name = cfg.name("net_pnl_change_1m_to_mcap");
        let mcap_version = Version::new(5);
        let mcap_source = all_chain.with_market_cap(
            &format!("{mcap_name}_ppm_source"),
            mcap_version,
            &core.net_pnl.delta.absolute._1m.cents.height,
            |_, net_pnl, market_cap| Self::net_pnl_to_market_cap(net_pnl, market_cap),
        );
        let net_pnl = RealizedNetPnl {
            change_1m_to_rcap: cfg.import("net_pnl_change_1m_to_rcap", Version::new(5))?,
            change_1m_to_mcap: LazyPercentPerBlock::from_uncached_height_source(
                &mcap_name,
                mcap_version,
                mcap_source,
                cfg.indexes,
            ),
        };

        // SOPR
        let sopr = RealizedSopr {
            ratio_extended: cfg.import("sopr", v1)?,
        };

        // Peak regret
        let peak_regret = RealizedPeakRegret {
            value: cfg.import("realized_peak_regret", Version::new(3))?,
        };

        // Capitalized
        let capitalized = RealizedCapitalized {
            price: cfg.import("capitalized_price", v0)?,
            cap_raw: cfg.import("capitalized_cap_raw", v0)?,
        };

        Ok(Self {
            core,
            cap_to_own_mcap,
            gross_pnl,
            sell_side_risk_ratio,
            net_pnl,
            sopr,
            peak_regret,
            capitalized,
            profit_to_loss_ratio: cfg.import("realized_profit_to_loss_ratio", v1)?,
            cap_raw: cfg.import("cap_raw", v0)?,
        })
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.capitalized
            .price
            .cents
            .height
            .len()
            .min(self.cap_raw.len())
            .min(self.capitalized.cap_raw.len())
            .min(self.peak_regret.value.cumulative.cents.height.len())
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
    pub(crate) fn push_state(
        &mut self,
        state: &CohortState<RealizedState, CostBasisData<WithCapital>>,
    ) {
        self.core.push_state(state);
        self.capitalized
            .price
            .cents
            .height
            .push(state.realized.capitalized_price());
        self.cap_raw.push(state.realized.cap_raw());
        self.capitalized
            .cap_raw
            .push(state.realized.capitalized_cap_raw());
        self.peak_regret
            .value
            .push_block(state.realized.peak_regret());
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.core.collect_vecs_mut();
        vecs.push(&mut self.capitalized.price.cents.height);
        vecs.push(&mut self.cap_raw as &mut dyn AnyStoredVec);
        vecs.push(&mut self.capitalized.cap_raw as &mut dyn AnyStoredVec);
        vecs.push(self.peak_regret.value.stored_mut());
        vecs
    }

    pub(crate) fn compute_from_stateful(
        &mut self,
        starting_lengths: &Lengths,
        others: &[&RealizedCore],
        exit: &Exit,
    ) -> Result<()> {
        self.core
            .compute_from_stateful(starting_lengths, others, exit)
    }

    #[inline(always)]
    pub(crate) fn push_accum(&mut self, accum: &RealizedFullAccum) -> Cents {
        self.cap_raw.push(accum.cap_raw);
        self.capitalized.cap_raw.push(accum.capitalized_cap_raw);

        let capitalized_price = {
            let cap = accum.cap_raw.as_u128();
            if cap == 0 {
                Cents::ZERO
            } else {
                Cents::new((accum.capitalized_cap_raw / cap) as u64)
            }
        };
        self.capitalized.price.cents.height.push(capitalized_price);

        self.peak_regret.value.push_block(accum.peak_regret());

        capitalized_price
    }

    pub(crate) fn compute_rest_part1(
        &mut self,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        self.core.compute_rest_part1(starting_lengths, exit)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_rest_part2(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        height_to_supply: &impl ReadableVec<Height, Bitcoin>,
        activity_transfer_volume: &ValuePerBlockCumulativeRolling,
        exit: &Exit,
    ) -> Result<()> {
        self.core.compute_rest_part2(
            prices,
            starting_lengths,
            height_to_supply,
            &activity_transfer_volume.sum._24h.cents.height,
            exit,
        )?;

        // SOPR ratios from lazy rolling sums (1w, 1m, 1y)
        self.sopr.ratio_extended.compute_columns2(
            starting_lengths.height,
            |window| {
                &window
                    .select_full(&activity_transfer_volume.sum.0)
                    .cents
                    .height
            },
            |window| {
                &window
                    .select_full(&self.core.sopr.value_destroyed.sum)
                    .height
            },
            |_, value_created, value_destroyed| RatioCents64::apply(value_created, value_destroyed),
            exit,
        )?;

        // Gross PnL
        self.gross_pnl.compute_from_cumulative_pair(
            starting_lengths.height,
            &self.core.minimal.profit.cumulative.cents.height,
            &self.core.minimal.loss.cumulative.cents.height,
            |_, profit, loss| profit + loss,
            exit,
        )?;

        // Net PnL 1m change relative to rcap and mcap
        self.net_pnl
            .change_1m_to_rcap
            .compute_binary::<CentsSigned, Cents, RatioCentsSignedCents<PartsPerMillionSigned64>>(
                starting_lengths.height,
                &self.core.net_pnl.delta.absolute._1m.cents.height,
                &self.core.minimal.cap.cents.height,
                exit,
            )?;
        // Sell-side risk ratios
        self.sell_side_risk_ratio.compute_columns2(
            starting_lengths.height,
            |window| &window.select(&self.gross_pnl.sum).cents.height,
            |_| &self.core.minimal.cap.cents.height,
            |_, realized_value, realized_cap| {
                RatioCents::<PartsPerMillion32>::apply(realized_value, realized_cap)
            },
            exit,
        )?;

        // Realized profit to loss ratios
        self.profit_to_loss_ratio.compute_columns2(
            starting_lengths.height,
            |window| &window.select(&self.core.minimal.profit.sum).cents.height,
            |window| &window.select(&self.core.minimal.loss.sum).cents.height,
            |_, profit, loss| RatioCents64::apply(profit, loss),
            exit,
        )?;

        Ok(())
    }
}

#[inline(always)]
fn mvrv_to_realized_cap_ratio(_: Height, mvrv: PartsPerMillion64) -> PartsPerMillion32 {
    PartsPerMillion32::from(1.0 / f64::from(mvrv))
}

#[derive(Default)]
pub struct RealizedFullAccum {
    pub(crate) cap_raw: CentsSats,
    pub(crate) capitalized_cap_raw: CentsSquaredSats,
    peak_regret: CentsSats,
}

impl RealizedFullAccum {
    pub(crate) fn add(&mut self, state: &RealizedState) {
        self.cap_raw += state.cap_raw();
        self.capitalized_cap_raw += state.capitalized_cap_raw();
        self.peak_regret += CentsSats::new(state.peak_regret_raw());
    }

    pub(crate) fn peak_regret(&self) -> Cents {
        self.peak_regret.to_cents()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realized_cap_ratio_is_inverse_mvrv() {
        assert_eq!(
            mvrv_to_realized_cap_ratio(Height::ZERO, PartsPerMillion64::from(2.0)),
            PartsPerMillion32::from(0.5),
        );
        assert_eq!(
            mvrv_to_realized_cap_ratio(Height::ZERO, PartsPerMillion64::from(1.0)),
            PartsPerMillion32::from(1.0),
        );
        assert!(mvrv_to_realized_cap_ratio(Height::ZERO, PartsPerMillion64::NAN).is_nan());
        assert!(mvrv_to_realized_cap_ratio(Height::ZERO, PartsPerMillion64::ZERO).is_nan());
    }
}
