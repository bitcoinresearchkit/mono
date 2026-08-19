use brk_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mining: &'a bitview_plugin_mining::Vecs,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
    pub market: &'a bitview_plugin_market::Vecs,
}
