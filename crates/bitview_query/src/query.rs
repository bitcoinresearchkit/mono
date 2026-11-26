use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[cfg(feature = "full-api")]
use arc_swap::ArcSwap;
#[cfg(feature = "series")]
use arc_swap::ArcSwapOption;
use bitview_plugin_indexer::Indexer;
#[cfg(feature = "full-api")]
use bitview_types::OraclePayments;
use bitview_types::SyncStatus;
use brk_mempool::ReadOnlyMempool;
#[cfg(feature = "full-api")]
use brk_mempool::ReadOnlyState;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::{BlockHash, Height, TxOut, Txid, Vout};
use rapidhash::RapidHashMap;
use vecdb::{ReadOnlyClone, ReadableVec, Ro};

use crate::{Error, OptionData, QueryPluginSet, Result, disk, plugins::QueryPlugins};
#[cfg(feature = "full-api")]
use crate::{
    PublicationCache, ReadPin,
    api::{StaleBlocks, oracle::OracleChunks},
};
#[cfg(feature = "series")]
use crate::{SeriesRegistry, pin::PublishedBounds};

/// Read-only queries whose resolved chain views pin the published prefix.
/// Bare lengths and unguarded internal helpers cannot authorize chain reads.
#[derive(Clone)]
pub struct Query(pub(crate) Arc<QueryInner<'static>>, pub(crate) Option<Instant>);
pub(crate) struct QueryInner<'a> {
    #[cfg(feature = "series")]
    registry: &'a SeriesRegistry<'a>,
    plugins: QueryPlugins<'a>,
    /// The node's tip height as the daemon last recorded it; 0 until then.
    node_height: AtomicU32,
    /// The bytes of the node's block files, recorded with the height.
    node_blocks_bytes: AtomicU64,
    /// The bytes Bitview's own data takes on disk as the daemon last measured them; 0 until then.
    data_bytes: AtomicU64,
    #[cfg(feature = "full-api")]
    mempool: Option<ReadOnlyMempool>,
    #[cfg(feature = "full-api")]
    pub(crate) live_oracle: PublicationCache<brk_oracle::Oracle>,
    #[cfg(feature = "full-api")]
    pub(crate) payment_chunks: parking_lot::Mutex<OracleChunks<OraclePayments>>,
    #[cfg(feature = "full-api")]
    pub(crate) stale_blocks: ArcSwap<StaleBlocks>,
    #[cfg(feature = "full-api")]
    /// Every committed audit summary, by height: replaced whole at each commit.
    pub(crate) audits: ArcSwap<Vec<brk_mempool::audit::Summary>>,
    /// Read bounds cached for the newest publication a pin asked for.
    #[cfg(feature = "series")]
    pub(crate) bounds: ArcSwapOption<PublishedBounds>,
}

impl Query {
    /// How long a read may wait for admission and run.
    pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(4);

    /// A cheap request-local view; shared data is unchanged.
    pub(crate) fn with_deadline(&self, deadline: Instant) -> Self {
        Self(Arc::clone(&self.0), Some(deadline))
    }

    pub(crate) fn check_deadline(&self) -> Result<()> {
        if self.1.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Error::ReadTimeout);
        }
        Ok(())
    }

    /// Builds the process-lifetime read-only query view.
    ///
    /// The cloned composition is intentionally leaked so query views can borrow
    /// it for the process lifetime. The series API also builds a catalog that
    /// borrows this composition. A daemon should call this once; repeated or
    /// multi-instance construction is outside this API's lifecycle contract.
    pub(crate) fn build<P>(plugins: &P, _mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        #[cfg(feature = "series")]
        let registry = Box::leak(Box::new(SeriesRegistry::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(
            Arc::new(QueryInner {
                #[cfg(feature = "series")]
                registry,
                plugins,
                node_height: AtomicU32::new(0),
                node_blocks_bytes: AtomicU64::new(0),
                data_bytes: AtomicU64::new(0),
                #[cfg(feature = "full-api")]
                mempool: _mempool,
                #[cfg(feature = "full-api")]
                live_oracle: Default::default(),
                #[cfg(feature = "full-api")]
                payment_chunks: Default::default(),
                #[cfg(feature = "full-api")]
                stale_blocks: Default::default(),
                #[cfg(feature = "full-api")]
                audits: Default::default(),
                #[cfg(feature = "series")]
                bounds: ArcSwapOption::empty(),
            }),
            None,
        )
    }

    /// Resolves the confirmed outputs mempool transactions spend, as published when called.
    pub fn indexer_prevout_resolver(
        &self,
    ) -> impl Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> + Send + Sync + use<> {
        let query = self.clone();
        // The reads check their own bounds: the pin alone, no calendar read bounds.
        move |holes| {
            let pin = query.indexer().pin();
            query.indexer().prevouts(holes, pin.lengths())
        }
    }

    /// Tip block hash at the pipeline-safe ceiling.
    #[inline]
    pub fn tip_blockhash(&self) -> BlockHash {
        self.indexer().tip_blockhash()
    }

    /// The published tip's height and hash, from one pin; `None` before genesis.
    pub fn tip_block(&self) -> Option<(Height, BlockHash)> {
        let pin = self.indexer().pin();
        pin.lengths()
            .last_height()
            .map(|height| (height, pin.tip()))
    }

    /// Records the node's tip height for [`Query::sync_status`] and the bytes of its block files
    /// for the disk usage. The daemon reads them before the server answers and when the node's
    /// tip changes, so no request asks the node.
    pub fn set_node_tip(&self, height: Height, blocks_bytes: u64) {
        self.0.node_height.store(*height, Ordering::Relaxed);
        self.0.node_blocks_bytes.store(blocks_bytes, Ordering::Relaxed);
    }

    /// The bytes of the node's block and undo files, as it last reported them.
    pub fn node_blocks_bytes(&self) -> u64 {
        self.0.node_blocks_bytes.load(Ordering::Relaxed)
    }

    /// Measures the bytes Bitview's data takes on disk and records them for
    /// [`Query::data_bytes`]. The daemon calls it before the server answers and after each
    /// publication, so no request reads the filesystem. The indexer stores' tables are summed
    /// from the sizes the stores track; the rest of `data_path` is walked, `logs_path` (the
    /// server's own logs, which grow with every request) left out.
    pub fn record_data_bytes(&self, data_path: &Path, logs_path: &Path) -> Result<()> {
        let (stores_path, stores_bytes) = self.indexer().stores().disk();
        // Left out by path, or the stores would be walked and counted twice.
        debug_assert!(stores_path.starts_with(data_path));
        let chain = self.reader().chain();
        // The default root can also hold each other chain's separate Bitview index.
        let nested = brk_types::Chain::ALL
            .iter()
            .filter(|other| other.network != chain.network && !other.subdir.is_empty())
            .map(|other| data_path.join(other.subdir))
            .collect::<Vec<_>>();
        let skip = [logs_path, stores_path]
            .into_iter()
            .chain(nested.iter().map(|path| path.as_path()))
            .collect::<Vec<_>>();
        let walked = disk::dir_size(data_path, &skip)?;
        let bytes = disk::sum_bytes(walked, stores_bytes)?;
        self.0.data_bytes.store(bytes, Ordering::Relaxed);
        Ok(())
    }

    /// The bytes Bitview's data takes on disk, as last measured.
    pub fn data_bytes(&self) -> u64 {
        self.0.data_bytes.load(Ordering::Relaxed)
    }

    /// The published index, from one safely published snapshot, against the node's tip as last
    /// recorded.
    pub fn sync_status(&self) -> Result<SyncStatus> {
        // Updating until the indexer's stores end their one merge after a catch-up: what waits for
        // the tip before it sends public traffic here waits for it too, while the
        // routes read the stores as they stand.
        if self.indexer().stores().merge_pending() {
            return Err(Error::updating());
        }
        let guard = self.pin_safe_lengths()?;
        let indexed_height = guard.lengths().last_height().ok_or_else(|| Error::updating())?;
        // An update can publish past the last record: the tip is never below what is indexed.
        let tip_height =
            Height::new(self.0.node_height.load(Ordering::Relaxed)).max(indexed_height);
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
    pub(crate) fn reader(&self) -> &Reader {
        self.indexer().reader()
    }

    #[inline]
    pub fn client(&self) -> &Client {
        self.reader().client()
    }

    #[inline]
    pub(crate) fn indexer(&self) -> &Indexer<Ro> {
        self.0.plugins.indexer
    }

    #[cfg(feature = "series")]
    #[inline]
    pub(crate) fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[cfg(feature = "full-api")]
    #[inline]
    pub(crate) fn mempool(&self) -> Option<Arc<ReadOnlyState>> {
        self.0.mempool.as_ref().map(ReadOnlyMempool::load)
    }

    #[cfg(feature = "full-api")]
    pub(crate) fn audit_reader(&self) -> Option<Arc<brk_mempool::audit::AuditReader>> {
        self.0.mempool.as_ref()?.audits()
    }

    /// The mempool, then a pin: the pool never follows a block before the chain publishes it, so
    /// loaded first it is at the pin's tip, behind it for the moment after blocks publish, or on
    /// the branch a reorg is replacing.
    #[cfg(feature = "full-api")]
    pub(crate) fn mempool_then_pin(&self) -> Result<(Option<Arc<ReadOnlyState>>, ReadPin)> {
        let mempool = self.mempool();
        Ok((mempool, self.pin_safe_lengths()?))
    }

    /// `mempool` at its own tip, for a lookup the chain answers first: whatever the chain holds
    /// answers before it, so the pool only adds what is still pending. Views adding the pool to
    /// the chain's numbers need it as the pin sees it instead ([`Query::with_pending`]).
    #[cfg(feature = "full-api")]
    pub(crate) fn lookup_mempool(
        mempool: Option<Arc<ReadOnlyState>>,
    ) -> Option<(Arc<ReadOnlyState>, BlockHash)> {
        mempool.map(|mempool| {
            // Before its first publication the pool has no tip, and its reads answer that.
            let tip = mempool.tip().unwrap_or_default();
            (mempool, tip)
        })
    }

    #[cfg(feature = "series")]
    #[inline]
    pub fn registry(&self) -> &'static SeriesRegistry<'static> {
        self.0.registry
    }
}
