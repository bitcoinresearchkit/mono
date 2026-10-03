use bitview_plugin_blocks::{HasBlocks, Vecs as Blocks};
use bitview_plugin_coinflow::{HasCoinflow, Vecs as Coinflow};
use bitview_plugin_cointime::{HasCointime, Vecs as Cointime};
use bitview_plugin_distribution_addresses::{
    HasDistributionAddresses, Vecs as DistributionAddresses,
};
use bitview_plugin_distribution_age::{HasDistributionAge, Vecs as DistributionAge};
use bitview_plugin_distribution_aggregated::{
    HasDistributionAggregated, Vecs as DistributionAggregated,
};
use bitview_plugin_distribution_utxos::{HasDistributionUtxos, Vecs as DistributionUtxos};
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_plugin_inputs::{HasInputs, Vecs as Inputs};
use bitview_plugin_mappings::{HasMappings, Vecs as Mappings};
use bitview_plugin_market::{HasMarket, Vecs as Market};
use bitview_plugin_mining::{HasMining, Vecs as Mining};
use bitview_plugin_outputs::{HasOutputs, Vecs as Outputs};
use bitview_plugin_pools::{HasPools, Vecs as Pools};
use bitview_plugin_price::{HasPrice, Vecs as Price};
use bitview_plugin_transactions::{HasTransactions, Vecs as Transactions};
use bitview_plugin_utxo_history::{HasUtxoHistory, Vecs as UtxoHistory};
use vecdb::StorageMode;

use crate::DefaultPlugins;

impl<M: StorageMode> HasIndexer<M> for DefaultPlugins<M> {
    fn indexer(&self) -> &Indexer<M> {
        self.indexer.as_ref()
    }
}

impl<M: StorageMode> HasBlocks<M> for DefaultPlugins<M> {
    fn blocks(&self) -> &Blocks<M> {
        self.blocks.as_ref()
    }
}

impl<M: StorageMode> HasMining<M> for DefaultPlugins<M> {
    fn mining(&self) -> &Mining<M> {
        self.mining.as_ref()
    }
}

impl<M: StorageMode> HasTransactions<M> for DefaultPlugins<M> {
    fn transactions(&self) -> &Transactions<M> {
        self.transactions.as_ref()
    }
}

impl<M: StorageMode> HasCointime<M> for DefaultPlugins<M> {
    fn cointime(&self) -> &Cointime<M> {
        self.cointime.as_ref()
    }
}

impl<M: StorageMode> HasCoinflow<M> for DefaultPlugins<M> {
    fn coinflow(&self) -> &Coinflow<M> {
        self.coinflow.as_ref()
    }
}

impl<M: StorageMode> HasMappings<M> for DefaultPlugins<M> {
    fn mappings(&self) -> &Mappings<M> {
        self.mappings.as_ref()
    }
}

impl<M: StorageMode> HasMarket<M> for DefaultPlugins<M> {
    fn market(&self) -> &Market<M> {
        self.market.as_ref()
    }
}

impl<M: StorageMode> HasPools<M> for DefaultPlugins<M> {
    fn pools(&self) -> &Pools<M> {
        self.pools.as_ref()
    }
}

impl<M: StorageMode> HasPrice<M> for DefaultPlugins<M> {
    fn price(&self) -> &Price<M> {
        self.price.as_ref()
    }
}

impl<M: StorageMode> HasDistributionAge<M> for DefaultPlugins<M> {
    fn distribution_age(&self) -> &DistributionAge<M> {
        self.distribution_age.as_ref()
    }
}

impl<M: StorageMode> HasInputs<M> for DefaultPlugins<M> {
    fn inputs(&self) -> &Inputs<M> {
        self.inputs.as_ref()
    }
}

impl<M: StorageMode> HasOutputs<M> for DefaultPlugins<M> {
    fn outputs(&self) -> &Outputs<M> {
        self.outputs.as_ref()
    }
}

impl<M: StorageMode> HasUtxoHistory<M> for DefaultPlugins<M> {
    fn utxo_history(&self) -> &UtxoHistory<M> {
        self.utxo_history.as_ref()
    }
}

impl<M: StorageMode> HasDistributionUtxos<M> for DefaultPlugins<M> {
    fn distribution_utxos(&self) -> &DistributionUtxos<M> {
        self.distribution_utxos.as_ref()
    }
}

impl<M: StorageMode> HasDistributionAddresses<M> for DefaultPlugins<M> {
    fn distribution_addresses(&self) -> &DistributionAddresses<M> {
        self.distribution_addresses.as_ref()
    }
}

impl<M: StorageMode> HasDistributionAggregated<M> for DefaultPlugins<M> {
    fn distribution_aggregated(&self) -> &DistributionAggregated<M> {
        self.distribution_aggregated.as_ref()
    }
}
