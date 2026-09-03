#[cfg(feature = "chain")]
mod addr;
#[cfg(feature = "chain")]
mod block;
#[cfg(feature = "chain")]
mod cached_json;
#[cfg(feature = "chain")]
mod cpfp;
mod indexer;
#[cfg(feature = "mappings")]
mod mappings;
#[cfg(feature = "chain")]
mod mempool;
#[cfg(feature = "chain")]
mod mining;
#[cfg(feature = "price")]
mod oracle;
#[cfg(feature = "price")]
mod price;
#[cfg(feature = "series")]
mod series;
#[cfg(feature = "chain")]
mod tx;
#[cfg(feature = "urpd")]
mod urpd;

#[cfg(feature = "mappings")]
use std::time::Duration;

#[cfg(feature = "mappings")]
use bitview_plugin::{Plugin, PluginReadGuard};
#[cfg(feature = "mappings")]
use brk_error::{Error, Result};

#[cfg(feature = "mappings")]
use crate::Query;

#[cfg(feature = "chain")]
pub(crate) use addr::{AddrMempoolTxsCache, AddrTxsCache};
#[cfg(feature = "chain")]
pub use addr::{
    AddrMempoolTxsPreflight, AddrTxsPreflight, ResolvedAddrChainTxs, ResolvedAddrMempoolTxs,
    ResolvedAddrTxs, ResolvedAddrUtxos,
};
#[cfg(feature = "chain")]
pub use block::{ResolvedBlock, ResolvedBlockTimestamp};
#[cfg(feature = "chain")]
pub(crate) use cached_json::{BoundedJsonCache, CachedJson};
#[cfg(feature = "chain")]
pub use cpfp::ResolvedCpfp;
#[cfg(feature = "chain")]
pub(crate) use mempool::BlockTemplateCache;
#[cfg(feature = "chain")]
pub use mempool::{BlockTemplateDiffPreflight, ResolvedRbf};
#[cfg(feature = "chain")]
pub use mining::ResolvedPoolBlocks;
#[cfg(feature = "price")]
pub use price::ResolvedHistoricalPrice;
#[cfg(feature = "series")]
pub use series::ResolvedQuery;
#[cfg(feature = "chain")]
pub use tx::{ResolvedConfirmedTx, ResolvedRawTransaction, ResolvedTransaction};
#[cfg(feature = "mappings")]
const UPDATE_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

#[cfg(feature = "mappings")]
impl Query {
    #[cfg(any(feature = "chain", feature = "price"))]
    fn read_plugin(&self, plugin: &impl Plugin) -> Result<PluginReadGuard> {
        plugin
            .gate()
            .read_for(UPDATE_WAIT_TIMEOUT)
            .ok_or(Error::StateUpdating)
    }

    #[cfg(feature = "chain")]
    fn try_read_plugin(&self, plugin: &impl Plugin) -> Option<PluginReadGuard> {
        plugin.gate().try_read()
    }

    fn read_plugins(&self, plugins: &[&dyn Plugin]) -> Result<PluginReadGuard> {
        PluginReadGuard::acquire_for(plugins, UPDATE_WAIT_TIMEOUT).ok_or(Error::StateUpdating)
    }
}
