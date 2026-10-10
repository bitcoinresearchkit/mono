use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::families::{CumulativeCount, CumulativeValue};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CoinDays;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    /// Value of outputs spent in each block. BTC representations use the spent
    /// output value; USD representations value it at the spending block's spot
    /// price.
    pub transfer_volume: CumulativeValue<M>,
    /// Transfer volume whose spending price is greater than or equal to the
    /// spent outputs' creation price.
    #[traversable(wrap = "transfer_volume", rename = "in_profit")]
    pub transfer_volume_in_profit: CumulativeValue<M>,
    /// Transfer volume whose spending price is below the spent outputs'
    /// creation price.
    #[traversable(wrap = "transfer_volume", rename = "in_loss")]
    pub transfer_volume_in_loss: CumulativeValue<M>,
    /// Coin days destroyed (CDD) by the cohort's outputs: each spent output's
    /// BTC value multiplied by its age in days.
    pub coindays_destroyed: CumulativeCount<CoinDays, M>,
}

impl ActivityVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        let version = version + Version::ONE;
        let value = |metric: &str| {
            CumulativeValue::import(db, &name(metric), version, mappings, window_starts)
        };
        Ok(Self {
            transfer_volume: value("transfer_volume")?,
            transfer_volume_in_profit: value("transfer_volume_in_profit")?,
            transfer_volume_in_loss: value("transfer_volume_in_loss")?,
            coindays_destroyed: CumulativeCount::import(
                db,
                &name("coindays_destroyed"),
                version,
                mappings,
                window_starts,
            )?,
        })
    }

    /// `(coin days destroyed, volume in profit, volume in loss)` and the total volume.
    #[inline(always)]
    pub fn push(
        &mut self,
        volume: Sats,
        (cdd, in_profit, in_loss): (CoinDays, Sats, Sats),
        price: Cents,
    ) {
        let cents = |sats: Sats| SatsToCents::apply(sats, price);
        self.transfer_volume.push_block(volume, cents(volume));
        self.transfer_volume_in_profit
            .push_block(in_profit, cents(in_profit));
        self.transfer_volume_in_loss
            .push_block(in_loss, cents(in_loss));
        self.coindays_destroyed.push_block(cdd);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.transfer_volume
            .stored_vecs_mut()
            .into_iter()
            .chain(self.transfer_volume_in_profit.stored_vecs_mut())
            .chain(self.transfer_volume_in_loss.stored_vecs_mut())
            .chain([self.coindays_destroyed.stored_mut()])
    }
}
