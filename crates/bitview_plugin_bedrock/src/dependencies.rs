use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, Ratio64};
use bitview_urpd::ReplayInputs;
use brk_types::Height;
use vecdb::ReadableVec;

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub distribution_aggregated: &'a AggregatedVecs,
    pub distribution_age: &'a AgeVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
}

impl Dependencies<'_> {
    pub(crate) fn raw_loss_share(&self) -> &impl ReadableVec<Height, PartsPerMillion32> {
        &self
            .distribution_aggregated
            .cohorts
            .all
            .relative
            .supply_in_loss_share
            .ppm
            .height
    }
    pub(crate) fn cointime_loss_share(&self) -> &impl ReadableVec<Height, Ratio64> {
        &self
            .cointime
            .supply
            .active_supply_in_loss_share
            .ratio
            .height
    }
    pub(crate) fn coinflow_loss_share(&self) -> &impl ReadableVec<Height, Ratio64> {
        &self.coinflow.all.supply_in_loss_share.height
    }
}
