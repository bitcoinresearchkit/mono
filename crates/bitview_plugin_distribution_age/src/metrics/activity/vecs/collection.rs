use bitview_cohort::CreationCohorts;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredF64;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, Rw, StorageMode};

use super::{CoindaysDestroyedByCohort, CumulativeValueByCohort};

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
    pub transfer_volume_in_profit: Box<CumulativeValueByCohort<M>>,
    #[traversable(wrap = "transfer_volume", rename = "in_loss")]
    /// Transfer volume whose spending price is below the spent outputs'
    /// creation price.
    pub transfer_volume_in_loss: Box<CumulativeValueByCohort<M>>,
}

impl ActivityVecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        let version = version + Version::ONE;
        let transfer_volume = Box::new(CumulativeValueByCohort::import(
            db,
            "transfer_volume",
            version,
            mappings,
            window_starts,
        )?);
        let coindays_destroyed =
            CoindaysDestroyedByCohort::import(db, version, mappings, window_starts)?;
        let transfer_volume_in_profit = Box::new(CumulativeValueByCohort::import(
            db,
            "transfer_volume_in_profit",
            version,
            mappings,
            window_starts,
        )?);
        let transfer_volume_in_loss = Box::new(CumulativeValueByCohort::import(
            db,
            "transfer_volume_in_loss",
            version,
            mappings,
            window_starts,
        )?);
        Ok(Box::new(Self {
            transfer_volume,
            coindays_destroyed,
            transfer_volume_in_profit,
            transfer_volume_in_loss,
        }))
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        height_price: Cents,
        transfer_volume: CreationCohorts<Sats>,
        coindays_destroyed: CreationCohorts<StoredF64>,
        transfer_volume_in_profit: CreationCohorts<Sats>,
        transfer_volume_in_loss: CreationCohorts<Sats>,
    ) {
        let transfer_value = transfer_volume.map(|sats| SatsToCents::apply(*sats, height_price));
        let profit_value =
            transfer_volume_in_profit.map(|sats| SatsToCents::apply(*sats, height_price));
        let loss_value =
            transfer_volume_in_loss.map(|sats| SatsToCents::apply(*sats, height_price));

        self.transfer_volume
            .push_block(&transfer_volume, &transfer_value);
        self.coindays_destroyed
            .stored
            .push_block(coindays_destroyed);
        self.transfer_volume_in_profit
            .push_block(&transfer_volume_in_profit, &profit_value);
        self.transfer_volume_in_loss
            .push_block(&transfer_volume_in_loss, &loss_value);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<_> = self.transfer_volume.stored_vecs_mut().collect();
        vecs.extend(self.coindays_destroyed.stored.stored_vecs_mut());
        vecs.extend(self.transfer_volume_in_profit.stored_vecs_mut());
        vecs.extend(self.transfer_volume_in_loss.stored_vecs_mut());
        vecs
    }
}
