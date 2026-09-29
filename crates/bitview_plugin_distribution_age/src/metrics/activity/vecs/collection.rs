use bitview_cohort::{CohortId, UTXOAggregate, UTXOAggregateId, UTXOCoreValues};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{DaysToYears, SatsToCents};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyWindowStartVec, RollingWindows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, StoredF32, StoredF64, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, Rw, StorageMode};

use super::{
    ActivitySources, CoindaysDestroyedByCohort, CoreCumulativeValueByCohort,
    CumulativeValueByCohort,
};

const COINYEARS_DESTROYED_VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    /// Value of outputs spent in each block. BTC representations use the spent
    /// output value; USD representations value it at the spending block's spot
    /// price.
    pub transfer_volume: Box<CumulativeValueByCohort<M>>,
    /// Coin days destroyed (CDD) by outputs from a UTXO cohort: each spent
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
    /// For each supported trailing window, average age in days of transferred
    /// bitcoin: coin days destroyed divided by transfer volume in BTC. Higher
    /// values mean older coins moved on average. Returns zero when transfer
    /// volume is zero.
    pub dormancy: UTXOAggregate<RollingWindows<StoredF32, M>>,
}

impl ActivityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        let aggregate_version = version;
        let version = version + Version::ONE;
        let transfer_volume = Box::new(CumulativeValueByCohort::forced_import(
            db,
            "transfer_volume",
            version,
            mappings,
            window_starts,
        )?);
        let coindays_destroyed =
            CoindaysDestroyedByCohort::forced_import(db, version, mappings, window_starts)?;
        let transfer_volume_in_profit = Box::new(CoreCumulativeValueByCohort::forced_import(
            db,
            "transfer_volume_in_profit",
            version,
            mappings,
            window_starts,
        )?);
        let transfer_volume_in_loss = Box::new(CoreCumulativeValueByCohort::forced_import(
            db,
            "transfer_volume_in_loss",
            version,
            mappings,
            window_starts,
        )?);
        let coinyears_destroyed = UTXOAggregate::from_fn(|id| {
            let cohort_id = id.cohort();
            let name = id.metric_name("coinyears_destroyed");
            let source = coindays_destroyed
                .cohorts
                .get(cohort_id)
                .expect("aggregate coindays-destroyed source")
                .sum
                ._1y
                .height
                .clone();
            LazyPerBlock::from_height_source::<DaysToYears>(
                &name,
                Self::aggregate_version(aggregate_version, id) + COINYEARS_DESTROYED_VERSION,
                &source,
                mappings,
            )
        });
        let dormancy = UTXOAggregate::try_from_fn(|id| {
            RollingWindows::forced_import(
                db,
                &id.metric_name("dormancy"),
                Self::aggregate_version(aggregate_version, id),
                mappings,
            )
        })?;
        Ok(Box::new(Self {
            transfer_volume,
            coindays_destroyed,
            transfer_volume_in_profit,
            transfer_volume_in_loss,
            coinyears_destroyed,
            dormancy,
        }))
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

    pub fn sources(&self, cohort_id: CohortId) -> Option<ActivitySources> {
        Some(ActivitySources {
            transfer_volume: self.transfer_volume.cohorts.get(cohort_id)?.clone(),
        })
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        height_price: Cents,
        transfer_volume: UTXOCoreValues<Sats>,
        coindays_destroyed: UTXOCoreValues<StoredF64>,
        transfer_volume_in_profit: UTXOCoreValues<Sats>,
        transfer_volume_in_loss: UTXOCoreValues<Sats>,
    ) {
        let transfer_value = transfer_volume.map(|sats| SatsToCents::apply(*sats, height_price));
        let profit_value =
            transfer_volume_in_profit.map(|sats| SatsToCents::apply(*sats, height_price));
        let loss_value =
            transfer_volume_in_loss.map(|sats| SatsToCents::apply(*sats, height_price));

        self.transfer_volume
            .push_block(transfer_volume, transfer_value);
        self.coindays_destroyed
            .stored
            .push_block(coindays_destroyed);
        self.transfer_volume_in_profit
            .push_block(transfer_volume_in_profit, profit_value);
        self.transfer_volume_in_loss
            .push_block(transfer_volume_in_loss, loss_value);
    }

    /// Dormancy is derived during post-processing and intentionally omitted.
    pub fn min_resume_len(&self) -> usize {
        self.transfer_volume
            .min_len()
            .min(self.coindays_destroyed.stored.min_len())
            .min(self.transfer_volume_in_profit.min_len())
            .min(self.transfer_volume_in_loss.min_len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.transfer_volume.collect_vecs_mut();
        vecs.extend(self.coindays_destroyed.stored.collect_vecs_mut());
        vecs.extend(self.transfer_volume_in_profit.collect_vecs_mut());
        vecs.extend(self.transfer_volume_in_loss.collect_vecs_mut());
        vecs.extend(
            self.dormancy
                .iter_mut()
                .flat_map(|value| value.as_mut_array())
                .map(|value| &mut value.height as &mut dyn AnyStoredVec),
        );
        vecs
    }

    pub fn compute_dormancy(&mut self, max_from: Height, exit: &Exit) -> Result<()> {
        for id in UTXOAggregateId::ALL {
            let cohort_id = id.cohort();
            let coindays_destroyed = &self
                .coindays_destroyed
                .cohorts
                .get(cohort_id)
                .expect("aggregate coindays-destroyed cohort")
                .sum;
            let transfer_volume = &self
                .transfer_volume
                .cohorts
                .get(cohort_id)
                .expect("aggregate transfer-volume cohort")
                .sum
                .0;
            for ((target, coindays), volume) in id
                .select_mut(&mut self.dormancy)
                .as_mut_array()
                .into_iter()
                .zip(coindays_destroyed.as_array())
                .zip(transfer_volume.as_array())
            {
                target.height.compute_transform2(
                    max_from,
                    &coindays.height,
                    &volume.btc.height,
                    |(height, rolling_coindays, rolling_btc, _)| {
                        let btc = f64::from(rolling_btc);
                        let value = if btc == 0.0 {
                            0.0
                        } else {
                            (f64::from(rolling_coindays) / btc) as f32
                        };
                        (height, StoredF32::from(value))
                    },
                    exit,
                )?;
            }
        }
        Ok(())
    }
}
