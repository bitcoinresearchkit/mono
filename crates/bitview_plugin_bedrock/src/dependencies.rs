use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_urpd::ReplayInputs;

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub holders: &'a HoldersVecs,
    pub age: &'a AgeVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
}
