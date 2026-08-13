use brk_cohort::{
    AmountRange, CohortContext, Filter, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES,
    UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF32, StoredF64, Version};
use vecdb::{AnyStoredVec, BinaryTransform, ColumnId, Database, Exit, Rw, StorageMode};

use crate::{
    distribution::metrics::UTXORows,
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarRollingWindows, Identity, LazyPerBlock, SatsToCents, Windows,
    },
};

use super::{
    ActivitySources, CoindaysDestroyedByCohort, CoreCumulativeValueByCohort,
    CumulativeValueByCohort,
};

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    pub transfer_volume: Box<CumulativeValueByCohort<M>>,
    pub coindays_destroyed: CoindaysDestroyedByCohort<M>,
    #[traversable(wrap = "transfer_volume", rename = "in_profit")]
    pub transfer_volume_in_profit: Box<CoreCumulativeValueByCohort<M>>,
    #[traversable(wrap = "transfer_volume", rename = "in_loss")]
    pub transfer_volume_in_loss: Box<CoreCumulativeValueByCohort<M>>,
    pub coinyears_destroyed: UTXOAggregate<LazyPerBlock<StoredF64, StoredF64>>,
    pub dormancy: UTXOAggregate<ColumnarRollingWindows<StoredF32, M>>,
}

impl ActivityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
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
            LazyPerBlock::from_height_source::<Identity<StoredF64>, _>(
                &name,
                Self::aggregate_version(aggregate_version, id),
                coindays_destroyed
                    .cohorts
                    .get(filter)
                    .expect("aggregate coindays-destroyed source")
                    .sum
                    ._1y
                    .height
                    .clone(),
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
