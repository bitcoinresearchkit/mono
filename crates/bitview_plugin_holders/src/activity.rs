use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{CoinDays, CoinYears, Days};
use bitview_transforms::DaysToYears;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazyPerBlockCumulativeRolling, LazyValuePerBlockCumulativeRolling,
    LazyWindowStartVec, RollingWindows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::columns::Columns;

#[derive(Traversable)]
pub struct Activity<M: StorageMode = Rw> {
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
    /// Transfer volume of outputs spent in profit.
    #[traversable(wrap = "transfer_volume", rename = "in_profit")]
    pub transfer_volume_in_profit: LazyValuePerBlockCumulativeRolling,
    /// Transfer volume of outputs spent in loss.
    #[traversable(wrap = "transfer_volume", rename = "in_loss")]
    pub transfer_volume_in_loss: LazyValuePerBlockCumulativeRolling,
    /// Coin days destroyed (CDD): spent coin amounts multiplied by their age in days.
    pub coindays_destroyed: LazyPerBlockCumulativeRolling<CoinDays>,
    pub coinyears_destroyed: LazyPerBlock<CoinYears, CoinDays>,
    /// Dormancy: coin days destroyed per spent coin over the window, the average age in days
    /// of the coins spent.
    pub dormancy: RollingWindows<Days, M>,
}
impl Activity {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let value = |metric: &str,
                     sats: &CachedSeries<Height, Sats>,
                     cents: &CachedSeries<Height, Cents>| {
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &id.metric_name(metric),
                v,
                sats,
                cents,
                mappings,
                windows,
            )
        };
        let coindays_destroyed = LazyPerBlockCumulativeRolling::from_cumulative_source(
            &id.metric_name("coindays_destroyed"),
            v,
            &c.cdd,
            windows,
            mappings,
        );
        let coinyears_destroyed = LazyPerBlock::from_height_source::<DaysToYears>(
            &id.metric_name("coinyears_destroyed"),
            v,
            &coindays_destroyed.sum._1y.height,
            mappings,
        );
        Ok(Self {
            transfer_volume: value("transfer_volume", &c.volume_sats, &c.volume_cents),
            transfer_volume_in_profit: value(
                "transfer_volume_in_profit",
                &c.volume_profit_sats,
                &c.volume_profit_cents,
            ),
            transfer_volume_in_loss: value(
                "transfer_volume_in_loss",
                &c.volume_loss_sats,
                &c.volume_loss_cents,
            ),
            coindays_destroyed,
            coinyears_destroyed,
            dormancy: RollingWindows::import(db, &id.metric_name("dormancy"), v, mappings)?,
        })
    }
    pub(crate) fn compute(&mut self, from: Height, exit: &Exit) -> Result<()> {
        for ((target, cdd), volume) in self
            .dormancy
            .as_mut_array()
            .into_iter()
            .zip(self.coindays_destroyed.sum.as_array())
            .zip(self.transfer_volume.sum.0.as_array())
        {
            target.height.compute_transform2(
                from,
                &cdd.height,
                &volume.btc.height,
                |(h, cdd, btc, _)| {
                    (
                        h,
                        Days::new(if f64::from(btc) == 0.0 {
                            0.0
                        } else {
                            (f64::from(cdd) / f64::from(btc)) as f32
                        }),
                    )
                },
                exit,
            )?;
        }
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.dormancy
            .as_mut_array()
            .into_iter()
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
    }
}
