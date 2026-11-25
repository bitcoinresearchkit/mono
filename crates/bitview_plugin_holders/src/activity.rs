use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{CoinDays, Days};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlockCumulativeRolling, LazyValuePerBlockCumulativeRolling,
    LazyWindowStartVec, RollingWindows, shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, Version};
use vecdb::{ComputeVec, Database, Rw, StorageMode};

use crate::columns::Columns;

#[derive(Traversable)]
#[traversable(client = "HoldersActivity")]
pub struct Activity<M: StorageMode = Rw> {
    /// Value of the cohort's outputs spent in each block. BTC representations use
    /// the spent output value; USD representations value it at the spending
    /// block's spot price.
    #[traversable(lead = "Transfer volume: the bitcoin spent.")]
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
    /// Value of the outputs spent in each block whose spending price is greater than or equal to
    /// their creation price; USD at the spending block's spot price.
    #[traversable(wrap = "transfer_volume", rename = "in_profit", priced)]
    #[traversable(lead = "Transfer volume in profit: the bitcoin spent at or above its cost.")]
    pub transfer_volume_in_profit: LazyValuePerBlockCumulativeRolling,
    /// Value of the outputs spent in each block whose spending price is below their creation
    /// price; USD at the spending block's spot price.
    #[traversable(wrap = "transfer_volume", rename = "in_loss", priced)]
    #[traversable(lead = "Transfer volume in loss: the bitcoin spent below its cost.")]
    pub transfer_volume_in_loss: LazyValuePerBlockCumulativeRolling,
    /// Coin days destroyed (CDD) sums each spent output's coin amount times its age in
    /// fractional days. The clock is the running maximum of block-header times.
    #[traversable(
        lead = "Coin days destroyed (CDD): coins spent, each times the days it was held, so old coins moving count most."
    )]
    pub coindays_destroyed: LazyPerBlockCumulativeRolling<CoinDays>,
    /// Dormancy is coin days destroyed per spent coin over the window. It uses the running
    /// maximum of block-header times and is undefined (null) when transfer volume is zero.
    #[traversable(lead = "Dormancy: how long the coins spent had been held, on average, in days.")]
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
        let coindays_destroyed = LazyPerBlockCumulativeRolling::from_cumulative_source(
            &id.metric_name("coindays_destroyed"),
            v,
            &c.cdd,
            windows,
            mappings,
        );
        // Every spent coin is in profit or in loss: the loss side's sats are what profit leaves.
        // Their cents are rounded per side, so they stay stored.
        let volume_loss_sats = LazyIndexedVec::new(
            &id.metric_name("transfer_volume_in_loss_cumulative_sats_source"),
            v,
            &c.volume_sats,
            &c.volume_profit_sats,
            |_, volume: Sats, in_profit: Sats| volume - in_profit,
        );
        Ok(Self {
            transfer_volume: LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &id.metric_name("transfer_volume"),
                v,
                &c.volume_sats,
                &c.volume_cents,
                mappings,
                windows,
            ),
            transfer_volume_in_profit: LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &id.metric_name("transfer_volume_in_profit"),
                v,
                &c.volume_profit_sats,
                &c.volume_profit_cents,
                mappings,
                windows,
            ),
            transfer_volume_in_loss: LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &id.metric_name("transfer_volume_in_loss"),
                v,
                &volume_loss_sats,
                &c.volume_loss_cents,
                mappings,
                windows,
            ),
            coindays_destroyed,
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
                |(_, cdd, btc, _)| Days::new((f64::from(cdd) / f64::from(btc)) as f32),
                exit,
            )?;
        }
        Ok(())
    }
}
