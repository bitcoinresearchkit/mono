use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{BoundedRatio, PartsPerMillion32};
use bitview_urpd::ReplayInputs;
use brk_types::Height;
use vecdb::ReadableVec;

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub holders: &'a HoldersVecs,
    pub age: &'a AgeVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
}

impl Dependencies<'_> {
    pub(crate) fn raw_loss_share(&self) -> &impl ReadableVec<Height, PartsPerMillion32> {
        &self.holders.cohorts.all.supply.in_loss.share.fixed.height
    }
    pub(crate) fn cointime_loss_share(&self) -> &impl ReadableVec<Height, BoundedRatio> {
        self.cointime.all_awake_supply_in_loss_share()
    }
    pub(crate) fn coinflow_loss_share(&self) -> &impl ReadableVec<Height, BoundedRatio> {
        self.coinflow.all_mobile_supply_in_loss_share()
    }
}
