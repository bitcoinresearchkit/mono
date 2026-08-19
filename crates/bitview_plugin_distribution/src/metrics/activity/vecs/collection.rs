use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{
    AmountRange, CohortContext, Filter, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES,
    UTXOAggregate, UTXOAggregateId,
};
use brk_types::{Cents, Height, Sats, StoredF32, StoredF64, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, ColumnId, Database, Exit, Rw, StorageMode, UnaryTransform,
};

use crate::metrics::UTXORows;
use bitview_compute::{
    CACHE_BUDGET, CachedWindowStartVec, ColumnarRollingWindows, LazyPerBlock, SatsToCents, Windows,
};

use super::{
    ActivitySources, CoindaysDestroyedByCohort, CoreCumulativeValueByCohort,
    CumulativeValueByCohort,
};

const COINYEARS_DESTROYED_VERSION: Version = Version::ONE;

struct CoinDaysToCoinYears;

impl UnaryTransform<StoredF64, StoredF64> for CoinDaysToCoinYears {
    #[inline(always)]
    fn apply(coin_days: StoredF64) -> StoredF64 {
        StoredF64::from(*coin_days / 365.0)
    }
}

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    /// Value of outputs from the selected cohort spent in each block. BTC
    /// representations use the spent output value; USD representations value
    /// it at the spending block's spot price.
    pub transfer_volume: Box<CumulativeValueByCohort<M>>,
    /// Coin days destroyed by outputs from the selected cohort: each spent
    /// output's BTC value multiplied by its age in days.
    pub coindays_destroyed: CoindaysDestroyedByCohort<M>,
    #[traversable(wrap = "transfer_volume", rename = "in_profit")]
    /// Transfer volume whose spending price is greater than or equal to the
    /// spent outputs' creation price.
    pub transfer_volume_in_profit: Box<CoreCumulativeValueByCohort<M>>,
    #[traversable(wrap = "transfer_volume", rename = "in_loss")]
    /// Transfer volume whose spending price is below the spent outputs'
    /// creation price.
    pub transfer_volume_in_loss: Box<CoreCumulativeValueByCohort<M>>,
    /// Coin years destroyed over the trailing 365-day window: the window's
    /// total coin days destroyed divided by 365.
    pub coinyears_destroyed: UTXOAggregate<LazyPerBlock<StoredF64, StoredF64>>,
    /// Average age in days of transferred bitcoin over the named trailing
    /// window: coin days destroyed divided by transfer volume in BTC.
    pub dormancy: UTXOAggregate<ColumnarRollingWindows<StoredF32, M>>,
}

impl ActivityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let aggregate_version = version;
        let version = version + Version::ONE;
        let transfer_volume = Box::new(CumulativeValueByCohort::forced_import(
            db,
            "transfer_volume",
            version,
            indexes,
            cached_starts,
        )?);
        let coindays_destroyed =
            CoindaysDestroyedByCohort::forced_import(db, version, indexes, cached_starts)?;
        let transfer_volume_in_profit = Box::new(CoreCumulativeValueByCohort::forced_import(
            db,
            "transfer_volume_in_profit",
            version,
            indexes,
            cached_starts,
        )?);
        let transfer_volume_in_loss = Box::new(CoreCumulativeValueByCohort::forced_import(
            db,
            "transfer_volume_in_loss",
            version,
            indexes,
            cached_starts,
        )?);
        let coinyears_destroyed = UTXOAggregate::from_fn(|id| {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            let name = Self::aggregate_metric_name(id, "coinyears_destroyed");
            let source = coindays_destroyed
                .cohorts
                .get(filter)
                .expect("aggregate coindays-destroyed source")
                .sum
                ._1y
                .height
                .clone();
            let source = CACHE_BUDGET.wrap(source);
            LazyPerBlock::from_height_source::<CoinDaysToCoinYears>(
                &name,
                Self::aggregate_version(aggregate_version, id) + COINYEARS_DESTROYED_VERSION,
                source,
                indexes,
            )
        });
        let dormancy = UTXOAggregate::try_from_fn(|id| {
            ColumnarRollingWindows::forced_import(
                db,
                &Self::aggregate_metric_name(id, "dormancy"),
                Self::aggregate_version(aggregate_version, id),
                indexes,
            )
        })?;
        Ok(Self {
            transfer_volume,
            coindays_destroyed,
            transfer_volume_in_profit,
            transfer_volume_in_loss,
            coinyears_destroyed,
            dormancy,
        })
    }

    fn aggregate_version(version: Version, id: UTXOAggregateId) -> Version {
        version
            + Version::ONE
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

    pub fn sources(&self, filter: &Filter) -> Option<ActivitySources> {
        Some(ActivitySources {
            transfer_volume: self.transfer_volume.cohorts.get(filter)?.clone(),
        })
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        height_price: Cents,
        transfer_volume: UTXORows<Sats>,
        coindays_destroyed: UTXORows<StoredF64>,
        transfer_volume_in_profit: UTXORows<Sats>,
        transfer_volume_in_loss: UTXORows<Sats>,
    ) {
        let transfer_value = transfer_volume.map(|sats| SatsToCents::apply(*sats, height_price));
        let profit_value =
            transfer_volume_in_profit.map(|sats| SatsToCents::apply(*sats, height_price));
        let loss_value =
            transfer_volume_in_loss.map(|sats| SatsToCents::apply(*sats, height_price));

        self.transfer_volume
            .push_block(transfer_volume, transfer_value);
        self.coindays_destroyed
            .cumulative
            .push_block(coindays_destroyed);
        self.transfer_volume_in_profit
            .push_block(transfer_volume_in_profit, profit_value);
        self.transfer_volume_in_loss
            .push_block(transfer_volume_in_loss, loss_value);
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, height_price: Cents, transfer_volume: &AmountRange<Sats>) {
        let cents = AmountRange::from_fn(|amount| {
            SatsToCents::apply(*amount.select(transfer_volume), height_price)
        });
        self.transfer_volume
            .push_addr_balance(transfer_volume, &cents);
    }

    /// Dormancy is derived during post-processing and intentionally omitted.
    pub fn min_resume_len(&self) -> usize {
        self.transfer_volume
            .min_len()
            .min(self.coindays_destroyed.cumulative.min_len())
            .min(self.transfer_volume_in_profit.min_len())
            .min(self.transfer_volume_in_loss.min_len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.transfer_volume.collect_vecs_mut();
        vecs.extend(self.coindays_destroyed.cumulative.collect_vecs_mut());
        vecs.extend(self.transfer_volume_in_profit.collect_vecs_mut());
        vecs.extend(self.transfer_volume_in_loss.collect_vecs_mut());
        vecs.extend(self.dormancy.iter_mut().map(|value| value.stored_mut()));
        vecs
    }

    pub fn compute_dormancy(&mut self, max_from: Height, exit: &Exit) -> Result<()> {
        for id in UTXOAggregateId::ALL {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            let coindays_destroyed = &self
                .coindays_destroyed
                .cohorts
                .get(filter)
                .expect("aggregate coindays-destroyed cohort")
                .sum;
            let transfer_volume = &self
                .transfer_volume
                .cohorts
                .get(filter)
                .expect("aggregate transfer-volume cohort")
                .sum
                .0;
            id.select_mut(&mut self.dormancy).compute_columns2(
                max_from,
                |window| &window.select(coindays_destroyed).height,
                |window| &window.select(transfer_volume).btc.height,
                |_, rolling_coindays, rolling_btc| {
                    let btc = f64::from(rolling_btc);
                    if btc == 0.0 {
                        StoredF32::from(0.0f32)
                    } else {
                        StoredF32::from((f64::from(rolling_coindays) / btc) as f32)
                    }
                },
                exit,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_trailing_coin_days_to_coin_years() {
        assert_eq!(
            CoinDaysToCoinYears::apply(StoredF64::from(365.0)),
            StoredF64::from(1.0),
        );
        assert_eq!(
            CoinDaysToCoinYears::apply(StoredF64::from(182.5)),
            StoredF64::from(0.5),
        );
        assert!(CoinDaysToCoinYears::apply(StoredF64::NAN).is_nan());
    }
}
