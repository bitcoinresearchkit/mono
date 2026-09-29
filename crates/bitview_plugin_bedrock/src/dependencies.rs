use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_urpd::ReplayInputs;
use brk_types::{Height, PartsPerMillion32, StoredF64};
use vecdb::ReadableVec;

use crate::{WeightedModeId, WeightedModes};

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub distribution_age: &'a AgeVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
}

impl Dependencies<'_> {
    pub(crate) fn raw_loss_share(&self) -> &impl ReadableVec<Height, PartsPerMillion32> {
        &self
            .distribution_age
            .cohorts
            .relative
            .supply_profitability_shares
            .supply_in_loss_share
            .all
            .ppm
            .height
    }
    pub(crate) fn weighted_loss_shares(
        &self,
    ) -> WeightedModes<&dyn ReadableVec<Height, StoredF64>> {
        WeightedModes::from_fn(|mode| -> &dyn ReadableVec<Height, StoredF64> {
            match mode {
                WeightedModeId::Cointime => {
                    &self
                        .cointime
                        .supply
                        .active_supply_in_loss_share
                        .ratio
                        .height
                }
                WeightedModeId::Coinflow => &self.coinflow.all.supply_in_loss_share.height,
                _ => {
                    &mode
                        .coinflow_horizon()
                        .unwrap()
                        .select(&self.coinflow.all.horizon)
                        .supply_in_loss_share
                        .height
                }
            }
        })
    }
}
