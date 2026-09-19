#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

#[cfg(feature = "indexer")]
use bitview_plugin_indexer::SafeLengths;

#[cfg(feature = "indexer")]
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(any(feature = "chain", feature = "series", feature = "price"))]
use bitview_plugin::PublicationReadGuard;

#[cfg(feature = "indexer")]
use bitview_plugin_indexer::{Indexer, Lengths};
#[cfg(feature = "indexer")]
use bitview_types::SyncStatus;
#[cfg(feature = "indexer")]
use brk_error::{Error, OptionData, Result};
#[cfg(feature = "indexer")]
use brk_mempool::{ReadOnlyMempool, ReadOnlyState};
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
#[cfg(feature = "price")]
mod live_oracle;
mod output;
#[cfg(feature = "indexer")]
mod query_plugin_set;
#[cfg(feature = "indexer")]
mod query_plugins;
mod representation_id;
mod series_output;
mod vecs;

#[cfg(feature = "indexer")]
mod r#impl;

#[cfg(feature = "tokio")]
pub use r#async::*;
#[cfg(feature = "urpd")]
pub use r#impl::ResolvedUrpd;
#[cfg(feature = "price")]
pub use r#impl::price::ResolvedHistoricalPrice;
#[cfg(feature = "chain")]
pub use r#impl::{
    BlockTemplateSource, ResolvedAddrChainTxs, ResolvedAddrTxs, ResolvedAddrUtxos,
    ResolvedBlockTemplateDiff, ResolvedBlockTimestamp, ResolvedBlocks, ResolvedBlocksV1,
    ResolvedConfirmedTx, ResolvedCpfp, ResolvedPoolBlocks, ResolvedRawTransaction, ResolvedRbf,
    ResolvedTransaction,
};
#[cfg(feature = "series")]
pub use r#impl::{ResolvedQuery, SeriesRead};
pub use output::*;
#[cfg(feature = "indexer")]
pub use query_plugin_set::{
    QueryPluginSet, SupportsBedrock, SupportsBlocks, SupportsCoinflow, SupportsCointime,
    SupportsDistribution, SupportsInputs, SupportsMappings, SupportsMining, SupportsOutputs,
    SupportsPools, SupportsPrice, SupportsTransactions,
};
#[cfg(feature = "indexer")]
pub use query_plugins::QueryPlugins;
pub use representation_id::RepresentationId;
pub use series_output::*;
pub use vecs::{ResolvedSeriesInfo, SeriesEntry, SeriesEntryLookup, Vecs};

#[cfg(feature = "indexer")]
/// Read-only queries whose resolved chain views pin the published prefix.
/// Bare lengths and unguarded internal helpers cannot authorize chain reads.
///
/// ```compile_fail
/// use bitview_query::Query;
/// use brk_types::{BlockHash, Height, Lengths};
/// fn unpinned(query: &Query, height: Height, hash: &BlockHash, safe: Lengths) {
///     query.block_raw_at_height(height, hash, safe).unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use bitview_query::Query;
/// use brk_types::{BlockHash, Height, Lengths};
/// fn unpinned(query: &Query, height: Height, hash: &BlockHash, safe: Lengths) {
///     query.block_raw_size_at_height(height, hash, safe).unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use bitview_query::Query;
/// use brk_types::{Height, Lengths};
/// fn unpinned(query: &Query, height: Height, lengths: Lengths) {
///     query.block_txids_by_height(height, lengths).unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use bitview_query::Query;
/// use brk_types::TxIndex;
/// fn unpinned(query: &Query, indices: &[TxIndex]) {
///     query.transactions_at_indices(indices).unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use bitview_query::Query;
/// use brk_types::Txid;
/// fn unguarded(query: &Query, txid: &Txid) {
///     query.resolve_confirmed_position(txid).unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct Query(Arc<QueryInner<'static>>, Option<Instant>);
#[cfg(feature = "indexer")]
struct QueryInner<'a> {
    vecs: &'a Vecs<'a>,
    plugins: QueryPlugins<'a>,
    mempool: Option<ReadOnlyMempool>,
    #[cfg(feature = "price")]
    live_oracle: live_oracle::LiveOracle,
}

#[cfg(feature = "indexer")]
impl Query {
    const UPDATE_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

    /// A cheap request-local view; shared data and publication guards are unchanged.
    pub fn with_deadline(&self, deadline: Instant) -> Self {
        Self(Arc::clone(&self.0), Some(deadline))
    }

    pub fn check_deadline(&self) -> Result<()> {
        if self.1.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Error::ReadTimeout);
        }
        Ok(())
    }

    fn read_timeout(&self) -> Result<Duration> {
        self.check_deadline()?;
        Ok(self.1.map_or(Self::UPDATE_WAIT_TIMEOUT, |deadline| {
            deadline.saturating_duration_since(Instant::now())
        }))
    }

    #[cfg(any(feature = "chain", feature = "series", feature = "price"))]
    fn read_publication(&self) -> Result<PublicationReadGuard> {
        self.indexer()
            .publication()
            .read_for(self.read_timeout()?)
            .ok_or(Error::ReadTimeout)
    }

    fn pin_safe_lengths(&self) -> Result<SafeLengths> {
        self.indexer()
            .pin_safe_lengths_for(self.read_timeout()?)
            .ok_or(Error::ReadTimeout)
    }

    #[cfg(feature = "chain")]
    fn try_read_publication(&self) -> Option<PublicationReadGuard> {
        self.indexer().publication().try_read()
    }

    /// Builds the process-lifetime read-only query view.
    ///
    /// The cloned composition and its vector catalog are intentionally leaked
    /// because the catalog contains references into that composition. A daemon
    /// should call this once; repeated or multi-instance query construction is
    /// outside this API's lifecycle contract.
    pub fn build<P>(plugins: &P, mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        let vecs = Box::leak(Box::new(Vecs::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(
            Arc::new(QueryInner {
                vecs,
                plugins,
                mempool,
                #[cfg(feature = "price")]
                live_oracle: Default::default(),
            }),
            None,
        )
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
    fn index_read_bounds(safe: Lengths) -> ReadBounds {
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

        bounds
    }

    #[cfg(feature = "series")]
    fn read_bounds(&self, safe: Lengths) -> ReadBounds {
        let mut bounds = Self::index_read_bounds(safe);
        let timestamp = safe.last_height().and_then(|height| {
            self.plugins()
                .mappings
                .timestamp
                .monotonic
                .collect_one(height)
        });
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

    /// Build sync status entirely from one safely published local snapshot.
    pub fn local_sync_status(&self) -> Result<SyncStatus> {
        self.sync_status_from(None)
    }

    /// Build sync status with the given external tip height. Both indexed and
    /// computed heights use one safely published pipeline snapshot.
    pub fn sync_status(&self, tip_height: Height) -> Result<SyncStatus> {
        self.sync_status_from(Some(tip_height))
    }

    fn sync_status_from(&self, tip_height: Option<Height>) -> Result<SyncStatus> {
        let guard = self.pin_safe_lengths()?;
        let indexed_height = guard.lengths().last_height().ok_or(Error::StateUpdating)?;
        let tip_height = tip_height.unwrap_or(indexed_height);
        let blocks_behind = Height::from(tip_height.saturating_sub(*indexed_height));
        let last_indexed_at_unix = self
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_one(indexed_height)
            .data()?;
        drop(guard);

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

    /// The shared read-only plugin composition backing this query view.
    #[inline]
    pub fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[inline]
    pub fn mempool(&self) -> Option<Arc<ReadOnlyState>> {
        self.0.mempool.as_ref().map(ReadOnlyMempool::load)
    }

    #[inline]
    pub fn vecs(&self) -> &'static Vecs<'static> {
        self.0.vecs
    }
}
