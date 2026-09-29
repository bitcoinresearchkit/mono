#[cfg(feature = "bedrock")]
use bitview_plugin_bedrock::HasBedrock;
#[cfg(feature = "blocks")]
use bitview_plugin_blocks::HasBlocks;
#[cfg(feature = "coinflow")]
use bitview_plugin_coinflow::HasCoinflow;
#[cfg(feature = "cointime")]
use bitview_plugin_cointime::HasCointime;
#[cfg(feature = "distribution_age")]
use bitview_plugin_distribution_age::HasDistributionAge;
#[cfg(feature = "distribution_size")]
use bitview_plugin_distribution_size::HasDistributionSize;
use bitview_plugin_indexer::HasIndexer;
#[cfg(feature = "inputs")]
use bitview_plugin_inputs::HasInputs;
#[cfg(feature = "mappings")]
use bitview_plugin_mappings::HasMappings;
#[cfg(feature = "mining")]
use bitview_plugin_mining::HasMining;
#[cfg(feature = "outputs")]
use bitview_plugin_outputs::HasOutputs;
#[cfg(feature = "pools")]
use bitview_plugin_pools::HasPools;
#[cfg(feature = "price")]
use bitview_plugin_price::HasPrice;
#[cfg(feature = "transactions")]
use bitview_plugin_transactions::HasTransactions;
#[cfg(feature = "utxo_history")]
use bitview_plugin_utxo_history::HasUtxoHistory;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use vecdb::Ro;

macro_rules! plugin_capability {
    ($feature:literal, $supports:ident, $has:ident) => {
        #[cfg(feature = $feature)]
        pub trait $supports: $has<Ro> {}

        #[cfg(feature = $feature)]
        impl<T> $supports for T where T: $has<Ro> {}

        #[cfg(not(feature = $feature))]
        pub trait $supports {}

        #[cfg(not(feature = $feature))]
        impl<T> $supports for T {}
    };
}

plugin_capability!("bedrock", SupportsBedrock, HasBedrock);
plugin_capability!("blocks", SupportsBlocks, HasBlocks);
plugin_capability!("coinflow", SupportsCoinflow, HasCoinflow);
plugin_capability!("cointime", SupportsCointime, HasCointime);
plugin_capability!(
    "distribution_size",
    SupportsDistributionSize,
    HasDistributionSize
);
plugin_capability!(
    "distribution_age",
    SupportsDistributionAge,
    HasDistributionAge
);
plugin_capability!("inputs", SupportsInputs, HasInputs);
plugin_capability!("mappings", SupportsMappings, HasMappings);
plugin_capability!("mining", SupportsMining, HasMining);
plugin_capability!("outputs", SupportsOutputs, HasOutputs);
plugin_capability!("pools", SupportsPools, HasPools);
plugin_capability!("price", SupportsPrice, HasPrice);
plugin_capability!("transactions", SupportsTransactions, HasTransactions);
plugin_capability!("utxo_history", SupportsUtxoHistory, HasUtxoHistory);

/// Composition contract for the individually enabled query plugins.
///
/// A composition whose query capabilities live on a nested plugin set can
/// return that set from [`query_capabilities`](Self::query_capabilities).
pub trait QueryPluginSet: PluginSet + Traversable {
    type Capabilities: HasIndexer<Ro>
        + SupportsBedrock
        + SupportsBlocks
        + SupportsCoinflow
        + SupportsCointime
        + SupportsDistributionAge
        + SupportsDistributionSize
        + SupportsInputs
        + SupportsMappings
        + SupportsMining
        + SupportsOutputs
        + SupportsPools
        + SupportsPrice
        + SupportsTransactions
        + SupportsUtxoHistory
        + ?Sized;

    fn query_capabilities(&self) -> &Self::Capabilities;
}

impl<T> QueryPluginSet for T
where
    T: PluginSet
        + Traversable
        + HasIndexer<Ro>
        + SupportsBedrock
        + SupportsBlocks
        + SupportsCoinflow
        + SupportsCointime
        + SupportsDistributionAge
        + SupportsDistributionSize
        + SupportsInputs
        + SupportsMappings
        + SupportsMining
        + SupportsOutputs
        + SupportsPools
        + SupportsPrice
        + SupportsTransactions
        + SupportsUtxoHistory,
{
    type Capabilities = Self;

    fn query_capabilities(&self) -> &Self::Capabilities {
        self
    }
}
