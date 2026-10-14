use crate::families::{CumulativeCount, CumulativeValue};
use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CoinDays;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, ops::SatsToCents, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{BinaryTransform, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct CreationActivity<M: StorageMode = Rw> {
    /// Value of outputs spent in each block. BTC representations use the spent
    /// output value; USD representations value it at the spending block's spot
    /// price.
    pub transfer_volume: CumulativeValue<M>,
    /// Coin days destroyed (CDD) by the cohort's outputs: each spent output's
    /// BTC value multiplied by its age in days.
    pub coindays_destroyed: CumulativeCount<CoinDays, M>,
}

impl CreationActivity {
    pub(super) fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        let version = version + Version::ONE;
        Ok(Self {
            transfer_volume: CumulativeValue::import(
                db,
                &name("transfer_volume"),
                version,
                mappings,
                window_starts,
            )?,
            coindays_destroyed: CumulativeCount::import(
                db,
                &name("coindays_destroyed"),
                version,
                mappings,
                window_starts,
            )?,
        })
    }

    #[inline(always)]
    pub(super) fn push(&mut self, volume: Sats, cdd: CoinDays, price: Cents) {
        self.transfer_volume
            .push_block(volume, SatsToCents::apply(volume, price));
        self.coindays_destroyed.push_block(cdd);
    }
}
