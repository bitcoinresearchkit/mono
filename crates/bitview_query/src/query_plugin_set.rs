#[cfg(feature = "urpd")]
use bitview_plugin_bedrock::HasBedrock;
#[cfg(feature = "chain")]
use bitview_plugin_blocks::HasBlocks;
#[cfg(feature = "urpd")]
use bitview_plugin_coinflow::HasCoinflow;
#[cfg(feature = "urpd")]
use bitview_plugin_cointime::HasCointime;
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_distribution::HasDistribution;
use bitview_plugin_indexer::HasIndexer;
#[cfg(any(feature = "chain", feature = "series"))]
use bitview_plugin_indexes::HasIndexes;
#[cfg(feature = "chain")]
use bitview_plugin_inputs::HasInputs;
#[cfg(feature = "chain")]
use bitview_plugin_mining::HasMining;
#[cfg(feature = "chain")]
use bitview_plugin_outputs::HasOutputs;
#[cfg(feature = "chain")]
use bitview_plugin_pools::HasPools;
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_price::HasPrice;
#[cfg(feature = "chain")]
use bitview_plugin_transactions::HasTransactions;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use vecdb::Ro;

#[cfg(feature = "chain")]
/// Capabilities required by chain queries.
pub trait SupportsChainQueries:
    HasBlocks<Ro>
    + HasDistribution<Ro>
    + HasIndexes<Ro>
    + HasInputs<Ro>
    + HasMining<Ro>
    + HasOutputs<Ro>
    + HasPools<Ro>
    + HasPrice<Ro>
    + HasTransactions<Ro>
{
}

#[cfg(feature = "chain")]
impl<T> SupportsChainQueries for T where
    T: HasBlocks<Ro>
        + HasDistribution<Ro>
        + HasIndexes<Ro>
        + HasInputs<Ro>
        + HasMining<Ro>
        + HasOutputs<Ro>
        + HasPools<Ro>
        + HasPrice<Ro>
        + HasTransactions<Ro>
{
}

#[cfg(not(feature = "chain"))]
/// No additional capabilities when chain queries are disabled.
pub trait SupportsChainQueries {}

#[cfg(not(feature = "chain"))]
impl<T> SupportsChainQueries for T {}

#[cfg(feature = "series")]
/// Capabilities required by series queries.
pub trait SupportsSeriesQueries: HasIndexes<Ro> {}

#[cfg(feature = "series")]
impl<T> SupportsSeriesQueries for T where T: HasIndexes<Ro> {}

#[cfg(not(feature = "series"))]
/// No additional capabilities when series queries are disabled.
pub trait SupportsSeriesQueries {}

#[cfg(not(feature = "series"))]
impl<T> SupportsSeriesQueries for T {}

#[cfg(feature = "urpd")]
/// Capabilities required by URPD queries.
pub trait SupportsUrpdQueries:
    HasBedrock<Ro> + HasCoinflow<Ro> + HasCointime<Ro> + HasDistribution<Ro> + HasPrice<Ro>
{
}

#[cfg(feature = "urpd")]
impl<T> SupportsUrpdQueries for T where
    T: HasBedrock<Ro> + HasCoinflow<Ro> + HasCointime<Ro> + HasDistribution<Ro> + HasPrice<Ro>
{
}

#[cfg(not(feature = "urpd"))]
/// No additional capabilities when URPD queries are disabled.
pub trait SupportsUrpdQueries {}

#[cfg(not(feature = "urpd"))]
impl<T> SupportsUrpdQueries for T {}

/// Complete composition contract for the enabled query features.
///
/// A composition whose query capabilities live on a nested plugin set can
/// return that set from [`query_capabilities`](Self::query_capabilities).
pub trait QueryPluginSet: PluginSet + Traversable {
    type Capabilities: HasIndexer<Ro>
        + SupportsChainQueries
        + SupportsSeriesQueries
        + SupportsUrpdQueries
        + ?Sized;

    fn query_capabilities(&self) -> &Self::Capabilities;
}

impl<T> QueryPluginSet for T
where
    T: PluginSet
        + Traversable
        + HasIndexer<Ro>
        + SupportsChainQueries
        + SupportsSeriesQueries
        + SupportsUrpdQueries,
{
    type Capabilities = Self;

    fn query_capabilities(&self) -> &Self::Capabilities {
        self
    }
}
