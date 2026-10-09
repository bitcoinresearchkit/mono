#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

use bitview_plugin_indexer::SafeLengths;

use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use bitview_plugin::Publication;
#[cfg(any(feature = "chain", feature = "series", feature = "price"))]
use bitview_plugin::PublicationReadGuard;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::BlockHashPrefix;
#[cfg(any(feature = "chain", feature = "series", feature = "price"))]
use bitview_primitives::Lengths;
#[cfg(feature = "series")]
use bitview_primitives::{Epoch, Halving, Index};
use bitview_types::SyncStatus;
use brk_mempool::ReadOnlyMempool;
#[cfg(any(feature = "chain", feature = "price"))]
use brk_mempool::ReadOnlyState;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::{BlockHash, Height};
#[cfg(feature = "series")]
use vecdb::ReadBounds;
use vecdb::{ReadOnlyClone, ReadableVec, Ro};

mod r#async;
mod error;
#[cfg(feature = "price")]
mod live_oracle;
mod output;
mod query_plugin_set;
mod query_plugins;
mod representation_id;
#[cfg(feature = "series")]
mod vecs;

mod r#impl;

pub use r#async::*;
pub(crate) use error::OptionData;
pub use error::{Error, Result, SeriesNotFound};
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
pub use query_plugin_set::{
    QueryPluginSet, SupportsAddresses, SupportsAge, SupportsBlocks, SupportsCoinflow,
    SupportsCointime, SupportsInputs, SupportsMappings, SupportsMining, SupportsOutputs,
    SupportsPools, SupportsPrice, SupportsTransactions, SupportsUtxoHistory,
};
pub use query_plugins::QueryPlugins;
pub use representation_id::RepresentationId;
#[cfg(feature = "series")]
pub use vecs::{ResolvedSeriesInfo, SeriesEntry, SharedSeries, Vecs};

/// Read-only queries whose resolved chain views pin the published prefix.
/// Bare lengths and unguarded internal helpers cannot authorize chain reads.
///
///
///
///
///
#[derive(Clone)]
pub struct Query(Arc<QueryInner<'static>>, Option<Instant>);
struct QueryInner<'a> {
    #[cfg(feature = "series")]
    vecs: &'a Vecs<'a>,
    plugins: QueryPlugins<'a>,
    #[cfg(any(feature = "chain", feature = "price"))]
    mempool: Option<ReadOnlyMempool>,
    #[cfg(feature = "price")]
    live_oracle: live_oracle::LiveOracle,
}

impl Query {
    const UPDATE_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

    /// A cheap request-local view; shared data and publication guards are unchanged.
    fn with_deadline(&self, deadline: Instant) -> Self {
        Self(Arc::clone(&self.0), Some(deadline))
    }

    fn check_deadline(&self) -> Result<()> {
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
    /// The cloned composition is intentionally leaked so query views can borrow
    /// it for the process lifetime. The series API also builds a catalog that
    /// borrows this composition. A daemon should call this once; repeated or
    /// multi-instance construction is outside this API's lifecycle contract.
    fn build<P>(plugins: &P, _mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        #[cfg(feature = "series")]
        let vecs = Box::leak(Box::new(Vecs::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(
            Arc::new(QueryInner {
                #[cfg(feature = "series")]
                vecs,
                plugins,
                #[cfg(any(feature = "chain", feature = "price"))]
                mempool: _mempool,
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
    #[cfg(feature = "chain")]
    fn height(&self) -> Height {
        self.safe_lengths().last_height().unwrap_or_default()
    }

    /// Snapshot of the pipeline-safe `Lengths`. Hot paths that need
    /// multiple bound fields should call this once at entry and reuse.
    #[cfg(any(feature = "chain", feature = "series", feature = "price"))]
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
    fn reader(&self) -> &Reader {
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
    fn indexer(&self) -> &Indexer<Ro> {
        self.0.plugins.indexer
    }

    /// Whether this query's mutable reads wait on `publication` (the indexer's gate).
    #[inline]
    pub fn reads_under(&self, publication: &Publication) -> bool {
        self.indexer().publication().ptr_eq(publication)
    }

    /// The shared read-only plugin composition backing this query view.
    #[cfg(any(feature = "chain", feature = "series", feature = "price"))]
    #[inline]
    fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[cfg(any(feature = "chain", feature = "price"))]
    #[inline]
    fn mempool(&self) -> Option<Arc<ReadOnlyState>> {
        self.0.mempool.as_ref().map(ReadOnlyMempool::load)
    }

    #[cfg(feature = "series")]
    #[inline]
    pub fn vecs(&self) -> &'static Vecs<'static> {
        self.0.vecs
    }
}
