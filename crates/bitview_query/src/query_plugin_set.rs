#[cfg(feature = "chain")]
use bitview_plugin_blocks::HasBlocks;
#[cfg(feature = "urpd")]
use bitview_plugin_coinflow::HasCoinflow;
#[cfg(feature = "urpd")]
use bitview_plugin_cointime::HasCointime;
#[cfg(feature = "chain")]
use bitview_plugin_distribution_addresses::HasDistributionAddresses;
#[cfg(feature = "urpd")]
use bitview_plugin_distribution_age::HasDistributionAge;
use bitview_plugin_indexer::HasIndexer;
#[cfg(feature = "chain")]
use bitview_plugin_inputs::HasInputs;
#[cfg(any(feature = "series", feature = "price"))]
use bitview_plugin_mappings::HasMappings;
#[cfg(feature = "chain")]
use bitview_plugin_mining::HasMining;
#[cfg(feature = "chain")]
use bitview_plugin_outputs::HasOutputs;
#[cfg(feature = "chain")]
use bitview_plugin_pools::HasPools;
#[cfg(feature = "price")]
use bitview_plugin_price::HasPrice;
#[cfg(feature = "chain")]
use bitview_plugin_transactions::HasTransactions;
#[cfg(any(feature = "chain", feature = "urpd"))]
use bitview_plugin_utxo_history::HasUtxoHistory;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use vecdb::Ro;

/// A plugin is read by the API features in `$cfg`; without them the capability is empty.
macro_rules! plugin_capability {
    (#[cfg($($cfg:tt)*)] $supports:ident, $has:ident) => {
        #[cfg($($cfg)*)]
        pub trait $supports: $has<Ro> {}

        #[cfg($($cfg)*)]
        impl<T> $supports for T where T: $has<Ro> {}

        #[cfg(not($($cfg)*))]
        pub trait $supports {}

        #[cfg(not($($cfg)*))]
        impl<T> $supports for T {}
    };
}

plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsBlocks,
    HasBlocks
);
plugin_capability!(
    #[cfg(feature = "urpd")]
    SupportsCoinflow,
    HasCoinflow
);
plugin_capability!(
    #[cfg(feature = "urpd")]
    SupportsCointime,
    HasCointime
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsDistributionAddresses,
    HasDistributionAddresses
);
plugin_capability!(
    #[cfg(feature = "urpd")]
    SupportsDistributionAge,
    HasDistributionAge
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsInputs,
    HasInputs
);
plugin_capability!(
    #[cfg(any(feature = "series", feature = "price"))]
    SupportsMappings,
    HasMappings
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsMining,
    HasMining
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsOutputs,
    HasOutputs
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsPools,
    HasPools
);
plugin_capability!(
    #[cfg(feature = "price")]
    SupportsPrice,
    HasPrice
);
plugin_capability!(
    #[cfg(feature = "chain")]
    SupportsTransactions,
    HasTransactions
);
plugin_capability!(
    #[cfg(any(feature = "chain", feature = "urpd"))]
    SupportsUtxoHistory,
    HasUtxoHistory
);

/// Composition contract for the plugins the enabled API features read.
///
/// A composition whose query capabilities live on a nested plugin set can
/// return that set from [`query_capabilities`](Self::query_capabilities).
pub trait QueryPluginSet: PluginSet + Traversable {
    type Capabilities: HasIndexer<Ro>
        + SupportsBlocks
        + SupportsCoinflow
        + SupportsCointime
        + SupportsDistributionAge
        + SupportsDistributionAddresses
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
        + SupportsBlocks
        + SupportsCoinflow
        + SupportsCointime
        + SupportsDistributionAge
        + SupportsDistributionAddresses
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
