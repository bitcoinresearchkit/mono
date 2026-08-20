use bitview_plugin_indexer::Indexer;

use bitview_plugin_distribution::UTXOStates;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a bitview_plugin_mappings::Vecs,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
    pub utxo_states: &'a UTXOStates,
    pub cointime: &'a bitview_plugin_cointime::Vecs,
    pub coinflow: &'a bitview_plugin_coinflow::Vecs,
}
