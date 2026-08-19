use bitview_plugin::Plugin;
use bitview_runtime::PluginSet;
use vecdb::Ro;

use crate::DefaultPlugins;

impl DefaultPlugins {
    fn plugins(&self) -> impl Iterator<Item = &dyn Plugin> {
        [
            self.indexer.as_ref() as &dyn Plugin,
            self.blocks.as_ref(),
            self.mining.as_ref(),
            self.transactions.as_ref(),
            self.cointime.as_ref(),
            self.coinflow.as_ref(),
            self.bedrock.as_ref(),
            self.capital_sentiment.as_ref(),
            self.rarity_meter.as_ref(),
            self.constants.as_ref(),
            self.indexes.as_ref(),
            self.indicators.as_ref(),
            self.investing.as_ref(),
            self.market.as_ref(),
            self.pools.as_ref(),
            self.price.as_ref(),
            self.distribution.as_ref(),
            self.supply.as_ref(),
            self.inputs.as_ref(),
            self.outputs.as_ref(),
            self.op_return.as_ref(),
        ]
        .into_iter()
    }
}

impl PluginSet for DefaultPlugins {
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin)) {
        self.plugins().for_each(visit);
    }
}

impl DefaultPlugins<Ro> {
    fn plugins(&self) -> impl Iterator<Item = &dyn Plugin> {
        [
            self.indexer.as_ref() as &dyn Plugin,
            self.blocks.as_ref(),
            self.mining.as_ref(),
            self.transactions.as_ref(),
            self.cointime.as_ref(),
            self.coinflow.as_ref(),
            self.bedrock.as_ref(),
            self.capital_sentiment.as_ref(),
            self.rarity_meter.as_ref(),
            self.constants.as_ref(),
            self.indexes.as_ref(),
            self.indicators.as_ref(),
            self.investing.as_ref(),
            self.market.as_ref(),
            self.pools.as_ref(),
            self.price.as_ref(),
            self.distribution.as_ref(),
            self.supply.as_ref(),
            self.inputs.as_ref(),
            self.outputs.as_ref(),
            self.op_return.as_ref(),
        ]
        .into_iter()
    }
}

impl PluginSet for DefaultPlugins<Ro> {
    fn for_each_plugin<'a>(&'a self, visit: &mut dyn FnMut(&'a dyn Plugin)) {
        self.plugins().for_each(visit);
    }
}
