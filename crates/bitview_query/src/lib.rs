#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

#[cfg(feature = "price")]
use std::sync::RwLock;
#[cfg(feature = "indexer")]
use std::{path::Path, sync::Arc};

#[cfg(feature = "bedrock")]
use bitview_plugin_bedrock::Vecs as Bedrock;
#[cfg(feature = "blocks")]
use bitview_plugin_blocks::Vecs as Blocks;
#[cfg(feature = "coinflow")]
use bitview_plugin_coinflow::Vecs as Coinflow;
#[cfg(feature = "cointime")]
use bitview_plugin_cointime::Vecs as Cointime;
#[cfg(feature = "distribution")]
use bitview_plugin_distribution::Vecs as Distribution;
#[cfg(feature = "indexer")]
use bitview_plugin_indexer::{Indexer, Lengths};
#[cfg(feature = "inputs")]
use bitview_plugin_inputs::Vecs as Inputs;
#[cfg(feature = "mappings")]
use bitview_plugin_mappings::Vecs as Mappings;
#[cfg(feature = "mining")]
use bitview_plugin_mining::Vecs as Mining;
#[cfg(feature = "outputs")]
use bitview_plugin_outputs::Vecs as Outputs;
#[cfg(feature = "pools")]
use bitview_plugin_pools::Vecs as Pools;
#[cfg(feature = "price")]
use bitview_plugin_price::Vecs as Price;
#[cfg(feature = "transactions")]
use bitview_plugin_transactions::Vecs as Transactions;
#[cfg(feature = "indexer")]
use bitview_types::SyncStatus;
#[cfg(feature = "indexer")]
use brk_error::{OptionData, Result};
#[cfg(feature = "indexer")]
use brk_mempool::Mempool;
#[cfg(feature = "price")]
use brk_oracle::Oracle;
#[cfg(feature = "indexer")]
use brk_reader::Reader;
#[cfg(feature = "indexer")]
use brk_rpc::Client;
#[cfg(feature = "indexer")]
use brk_types::{BlockHash, BlockHashPrefix, Height};
#[cfg(feature = "series")]
use brk_types::{Epoch, Halving, Index};
#[cfg(feature = "series")]
use vecdb::ReadBounds;
#[cfg(feature = "indexer")]
use vecdb::{ReadOnlyClone, ReadableVec, Ro};

#[cfg(feature = "tokio")]
mod r#async;
mod output;
#[cfg(feature = "indexer")]
mod query_plugin_set;
#[cfg(feature = "indexer")]
mod query_plugins;
mod series_output;
mod series_output_legacy;
mod vecs;

#[cfg(feature = "indexer")]
mod r#impl;

#[cfg(feature = "tokio")]
pub use r#async::*;
#[cfg(feature = "chain")]
pub use r#impl::AddrStatsPreflight;
#[cfg(feature = "series")]
pub use r#impl::ResolvedQuery;
pub use output::*;
#[cfg(feature = "indexer")]
pub use query_plugin_set::{
    QueryPluginSet, SupportsBedrock, SupportsBlocks, SupportsChainQueries, SupportsCoinflow,
    SupportsCointime, SupportsDistribution, SupportsInputs, SupportsMappings, SupportsMining,
    SupportsOutputs, SupportsPools, SupportsPrice, SupportsSeriesQueries, SupportsTransactions,
    SupportsUrpdQueries,
};
pub use series_output::*;
pub use series_output_legacy::*;
pub use vecs::Vecs;

#[cfg(feature = "indexer")]
use query_plugins::QueryPlugins;

#[cfg(feature = "indexer")]
#[derive(Clone)]
pub struct Query(Arc<QueryInner<'static>>);
#[cfg(feature = "indexer")]
struct QueryInner<'a> {
    vecs: &'a Vecs<'a>,
    plugins: QueryPlugins<'a>,
    mempool: Option<Mempool>,
    #[cfg(feature = "price")]
    live_oracle: RwLock<Option<(Height, Arc<Oracle>)>>,
}

#[cfg(feature = "indexer")]
impl Query {
    /// Builds the process-lifetime read-only query view.
    ///
    /// The cloned composition and its vector catalog are intentionally leaked
    /// because the catalog contains references into that composition. A daemon
    /// should call this once; repeated or multi-instance query construction is
    /// outside this API's lifecycle contract.
    pub fn build<P>(plugins: &P, mempool: Option<Mempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        let vecs = Box::leak(Box::new(Vecs::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(Arc::new(QueryInner {
            vecs,
            plugins,
            mempool,
            #[cfg(feature = "price")]
            live_oracle: RwLock::new(None),
        }))
    }

    /// Pipeline-safe ceiling: the highest height for which the complete
    /// plugin set has committed durable data. Backed by
    /// `Indexer::safe_lengths()`, advanced after each complete compute
    /// pass and lowered before any rollback.
    ///
    /// Returns a height (the last fully-written block), not a length.
    /// `safe_lengths().height` is a count: `N` means heights `0..N` are
    /// committed, so the highest is `N-1`. Pre-genesis (`N == 0`) falls
    /// back to `Height::default()` and clients treat it as "nothing
    /// indexed yet".
    pub fn height(&self) -> Height {
        self.safe_lengths().last_height().unwrap_or_default()
    }

    /// Snapshot of the pipeline-safe `Lengths`. Hot paths that need
    /// multiple bound fields should call this once at entry and reuse.
    fn safe_lengths(&self) -> Lengths {
        self.indexer().safe_lengths()
    }

    #[cfg(feature = "series")]
    fn read_bounds(&self, safe: Lengths) -> ReadBounds {
        let mut bounds = ReadBounds::new();

        bounds.set(Index::Height.name(), safe.height.into());
        bounds.set(Index::TxIndex.name(), safe.tx_index.into());
        bounds.set(Index::TxInIndex.name(), safe.txin_index.into());
        bounds.set(Index::TxOutIndex.name(), safe.txout_index.into());
        bounds.set(
            Index::EmptyOutputIndex.name(),
            safe.empty_output_index.into(),
        );
        bounds.set(Index::OpReturnIndex.name(), safe.op_return_index.into());
        bounds.set(Index::P2AAddrIndex.name(), safe.p2a_addr_index.into());
        bounds.set(Index::P2MSOutputIndex.name(), safe.p2ms_output_index.into());
        bounds.set(Index::P2PK33AddrIndex.name(), safe.p2pk33_addr_index.into());
        bounds.set(Index::P2PK65AddrIndex.name(), safe.p2pk65_addr_index.into());
        bounds.set(Index::P2PKHAddrIndex.name(), safe.p2pkh_addr_index.into());
        bounds.set(Index::P2SHAddrIndex.name(), safe.p2sh_addr_index.into());
        bounds.set(Index::P2TRAddrIndex.name(), safe.p2tr_addr_index.into());
        bounds.set(Index::P2WPKHAddrIndex.name(), safe.p2wpkh_addr_index.into());
        bounds.set(Index::P2WSHAddrIndex.name(), safe.p2wsh_addr_index.into());
        bounds.set(
            Index::UnknownOutputIndex.name(),
            safe.unknown_output_index.into(),
        );

        let tip = safe.last_height();
        bounds.set(
            Index::Epoch.name(),
            tip.map(|height| usize::from(Epoch::from(height)) + 1)
                .unwrap_or(0),
        );
        bounds.set(
            Index::Halving.name(),
            tip.map(|height| usize::from(Halving::from(height)) + 1)
                .unwrap_or(0),
        );

        let timestamp =
            tip.and_then(|height| self.indexer().vecs().blocks.timestamp.collect_one(height));
        for index in Index::all().into_iter().filter(Index::is_date_based) {
            let len = timestamp
                .and_then(|timestamp| index.timestamp_to_index(timestamp))
                .map(|last| last + 1)
                .unwrap_or(0);
            bounds.set(index.name(), len);
        }

        bounds
    }

    /// Tip block hash at the pipeline-safe ceiling.
    #[inline]
    pub fn tip_blockhash(&self) -> BlockHash {
        self.indexer().tip_blockhash()
    }

    /// Tip block hash prefix for cache etags.
    #[inline]
    pub fn tip_hash_prefix(&self) -> BlockHashPrefix {
        BlockHashPrefix::from(&self.tip_blockhash())
    }

    /// Build sync status with the given tip height. Both indexed and computed
    /// heights use the safely published pipeline ceiling.
    pub fn sync_status(&self, tip_height: Height) -> Result<SyncStatus> {
        let safe = self.safe_lengths();
        let indexed_height = safe.last_height().unwrap_or_default();
        let blocks_behind = Height::from(tip_height.saturating_sub(*indexed_height));
        let last_indexed_at_unix = self
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_one(indexed_height)
            .data()?;

        Ok(SyncStatus {
            indexed_height,
            computed_height: indexed_height,
            tip_height,
            blocks_behind,
            last_indexed_at: last_indexed_at_unix.to_iso8601(),
            last_indexed_at_unix,
        })
    }

    #[inline]
    pub fn reader(&self) -> &Reader {
        self.indexer().reader()
    }

    #[inline]
    pub fn client(&self) -> &Client {
        self.reader().client()
    }

    #[inline]
    pub fn blocks_dir(&self) -> &Path {
        self.reader().blocks_dir()
    }

    #[inline]
    pub fn indexer(&self) -> &Indexer<Ro> {
        self.0.plugins.indexer
    }

    #[cfg(feature = "bedrock")]
    #[inline]
    pub fn bedrock(&self) -> &Bedrock<Ro> {
        self.0.plugins.bedrock
    }

    #[cfg(feature = "blocks")]
    #[inline]
    pub fn blocks_plugin(&self) -> &Blocks<Ro> {
        self.0.plugins.blocks
    }

    #[cfg(feature = "coinflow")]
    #[inline]
    pub fn coinflow(&self) -> &Coinflow<Ro> {
        self.0.plugins.coinflow
    }

    #[cfg(feature = "cointime")]
    #[inline]
    pub fn cointime(&self) -> &Cointime<Ro> {
        self.0.plugins.cointime
    }

    #[cfg(feature = "distribution")]
    #[inline]
    pub fn distribution(&self) -> &Distribution<Ro> {
        self.0.plugins.distribution
    }

    #[cfg(feature = "inputs")]
    #[inline]
    pub fn inputs(&self) -> &Inputs<Ro> {
        self.0.plugins.inputs
    }

    #[cfg(feature = "mappings")]
    #[inline]
    pub fn mappings(&self) -> &Mappings<Ro> {
        self.0.plugins.mappings
    }

    #[cfg(feature = "mining")]
    #[inline]
    pub fn mining(&self) -> &Mining<Ro> {
        self.0.plugins.mining
    }

    #[cfg(feature = "outputs")]
    #[inline]
    pub fn outputs(&self) -> &Outputs<Ro> {
        self.0.plugins.outputs
    }

    #[inline]
    #[cfg(any(
        feature = "chain",
        feature = "series",
        feature = "urpd",
        feature = "price"
    ))]
    fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[cfg(feature = "pools")]
    #[inline]
    pub fn pools(&self) -> &Pools<Ro> {
        self.0.plugins.pools
    }

    #[cfg(feature = "price")]
    #[inline]
    pub fn price(&self) -> &Price<Ro> {
        self.0.plugins.price
    }

    #[cfg(feature = "transactions")]
    #[inline]
    pub fn transactions(&self) -> &Transactions<Ro> {
        self.0.plugins.transactions
    }

    #[inline]
    pub fn mempool(&self) -> Option<&Mempool> {
        self.0.mempool.as_ref()
    }

    #[inline]
    pub fn vecs(&self) -> &'static Vecs<'static> {
        self.0.vecs
    }
}
