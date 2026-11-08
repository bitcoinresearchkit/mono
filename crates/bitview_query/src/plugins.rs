use bitview_plugin::PluginSet;
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_traversable::Traversable;
use vecdb::Ro;

/// The plugins the query layer reads beyond the indexer, each under the feature whose endpoints
/// read it: its typed reference on [`QueryPlugins`] and its requirement on [`QueryPluginSet`]. A
/// composition without the feature doesn't need the plugin.
macro_rules! query_plugins {
    ($($feature:tt $field:ident: $plugin:ident::$has:ident as $supports:ident,)*) => {
        $(
            #[cfg(feature = $feature)]
            pub trait $supports: $plugin::$has<Ro> {}

            #[cfg(feature = $feature)]
            impl<T> $supports for T where T: $plugin::$has<Ro> {}

            #[cfg(not(feature = $feature))]
            pub trait $supports {}

            #[cfg(not(feature = $feature))]
            impl<T> $supports for T {}
        )*

        /// Direct typed references, so hot paths perform no dynamic lookup.
        pub(crate) struct QueryPlugins<'a> {
            pub(crate) indexer: &'a Indexer<Ro>,
            $(
                #[cfg(feature = $feature)]
                pub(crate) $field: &'a $plugin::Vecs<Ro>,
            )*
        }

        impl<'a> QueryPlugins<'a> {
            pub(crate) fn new<P>(plugins: &'a P) -> Self
            where
                P: QueryPluginSet,
            {
                let plugins = plugins.query_capabilities();
                Self {
                    indexer: plugins.indexer(),
                    $(
                        #[cfg(feature = $feature)]
                        $field: $plugin::$has::<Ro>::$field(plugins),
                    )*
                }
            }
        }

        /// Composition contract for the plugins the enabled API features read.
        ///
        /// A composition whose query capabilities live on a nested plugin set can
        /// return that set from [`query_capabilities`](Self::query_capabilities).
        pub trait QueryPluginSet: PluginSet + Traversable {
            type Capabilities: HasIndexer<Ro> $(+ $supports)* + ?Sized;

            fn query_capabilities(&self) -> &Self::Capabilities;
        }

        impl<T> QueryPluginSet for T
        where
            T: PluginSet + Traversable + HasIndexer<Ro> $(+ $supports)*,
        {
            type Capabilities = Self;

            fn query_capabilities(&self) -> &Self::Capabilities {
                self
            }
        }
    };
}

query_plugins! {
    "series" mappings: bitview_plugin_mappings::HasMappings as SupportsMappings,
    "full-api" addresses: bitview_plugin_addresses::HasAddresses as SupportsAddresses,
    "full-api" age: bitview_plugin_age::HasAge as SupportsAge,
    "full-api" blocks: bitview_plugin_blocks::HasBlocks as SupportsBlocks,
    "full-api" coinflow: bitview_plugin_coinflow::HasCoinflow as SupportsCoinflow,
    "full-api" cointime: bitview_plugin_cointime::HasCointime as SupportsCointime,
    "full-api" inputs: bitview_plugin_inputs::HasInputs as SupportsInputs,
    "full-api" mining: bitview_plugin_mining::HasMining as SupportsMining,
    "full-api" outputs: bitview_plugin_outputs::HasOutputs as SupportsOutputs,
    "full-api" pools: bitview_plugin_pools::HasPools as SupportsPools,
    "full-api" price: bitview_plugin_price::HasPrice as SupportsPrice,
    "full-api" silent_payments: bitview_plugin_silent_payments::HasSilentPayments as SupportsSilentPayments,
    "full-api" transactions: bitview_plugin_transactions::HasTransactions as SupportsTransactions,
    "full-api" utxo_set: bitview_plugin_utxo_set::HasUtxoSet as SupportsUtxoSet,
}
