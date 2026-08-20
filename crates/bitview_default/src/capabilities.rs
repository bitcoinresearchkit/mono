use bitview_plugin_bedrock::{HasBedrock, Vecs as Bedrock};
use bitview_plugin_blocks::{HasBlocks, Vecs as Blocks};
use bitview_plugin_capital_sentiment::{HasCapitalSentiment, Vecs as CapitalSentiment};
use bitview_plugin_coinflow::{HasCoinflow, Vecs as Coinflow};
use bitview_plugin_cointime::{HasCointime, Vecs as Cointime};
use bitview_plugin_constants::{HasConstants, Vecs as Constants};
use bitview_plugin_distribution::{HasDistribution, Vecs as Distribution};
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_plugin_indicators::{HasIndicators, Vecs as Indicators};
use bitview_plugin_inputs::{HasInputs, Vecs as Inputs};
use bitview_plugin_investing::{HasInvesting, Vecs as Investing};
use bitview_plugin_mappings::{HasMappings, Vecs as Mappings};
use bitview_plugin_market::{HasMarket, Vecs as Market};
use bitview_plugin_mining::{HasMining, Vecs as Mining};
use bitview_plugin_op_return::{HasOpReturn, Vecs as OpReturn};
use bitview_plugin_outputs::{HasOutputs, Vecs as Outputs};
use bitview_plugin_pools::{HasPools, Vecs as Pools};
use bitview_plugin_price::{HasPrice, Vecs as Price};
use bitview_plugin_rarity_meter::{HasRarityMeter, Vecs as RarityMeter};
use bitview_plugin_supply::{HasSupply, Vecs as Supply};
use bitview_plugin_transactions::{HasTransactions, Vecs as Transactions};
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

impl<M: StorageMode> HasBedrock<M> for DefaultPlugins<M> {
    fn bedrock(&self) -> &Bedrock<M> {
        self.bedrock.as_ref()
    }
}

impl<M: StorageMode> HasCapitalSentiment<M> for DefaultPlugins<M> {
    fn capital_sentiment(&self) -> &CapitalSentiment<M> {
        self.capital_sentiment.as_ref()
    }
}

impl<M: StorageMode> HasRarityMeter<M> for DefaultPlugins<M> {
    fn rarity_meter(&self) -> &RarityMeter<M> {
        self.rarity_meter.as_ref()
    }
}

impl<M: StorageMode> HasConstants for DefaultPlugins<M> {
    fn constants(&self) -> &Constants {
        self.constants.as_ref()
    }
}

impl<M: StorageMode> HasMappings<M> for DefaultPlugins<M> {
    fn mappings(&self) -> &Mappings<M> {
        self.mappings.as_ref()
    }
}

impl<M: StorageMode> HasIndicators<M> for DefaultPlugins<M> {
    fn indicators(&self) -> &Indicators<M> {
        self.indicators.as_ref()
    }
}

impl<M: StorageMode> HasInvesting for DefaultPlugins<M> {
    fn investing(&self) -> &Investing {
        self.investing.as_ref()
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

impl<M: StorageMode> HasDistribution<M> for DefaultPlugins<M> {
    fn distribution(&self) -> &Distribution<M> {
        self.distribution.as_ref()
    }
}

impl<M: StorageMode> HasSupply<M> for DefaultPlugins<M> {
    fn supply(&self) -> &Supply<M> {
        self.supply.as_ref()
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

impl<M: StorageMode> HasOpReturn<M> for DefaultPlugins<M> {
    fn op_return(&self) -> &OpReturn<M> {
        self.op_return.as_ref()
    }
}
